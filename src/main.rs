use crate::prometheus::{QueryResult, RangeQuery};
use axum::response::IntoResponse;
use axum::{response::Html, routing::get, Router};
use clap::Parser;
use futures_util::stream::FuturesUnordered;
use futures_util::stream::{StreamExt, TryStreamExt};
use poloto::num::timestamp::UnixTime;
use reqwest::StatusCode;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_livereload::LiveReloadLayer;

mod prometheus;

const ONE_HOUR_IN_SECS: u64 = 3600;

fn yesterday() -> u64 {
    now() - ONE_HOUR_IN_SECS * 24
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Generate a plot from a Prometheus range query.
#[derive(Parser, Debug, Clone)]
struct Opts {
    /// The Prometheus server endpoint.
    #[arg(short, long, default_value = "http://localhost:9090")]
    endpoint: String,

    /// The start of the query range (UNIX timestamp)
    ///
    /// Defaults to 24 hours ago.
    #[arg(long, default_value_t = yesterday())]
    start: u64,

    /// The end of the query range (UNIX timestamp)
    ///
    /// Defaults to now.
    #[arg(long, default_value_t = now())]
    end: u64,

    /// The range query step (in seconds)
    #[arg(long, default_value_t = 60)]
    step: u64,

    /// The title of the plot.
    ///
    /// Multiple titles can be specified by passing the flag multiple times.
    ///
    /// The number of titles must match the number of queries.
    #[arg(short, long)]
    title: Vec<String>,

    /// Open the plot in the browser and live-reload it.
    #[arg(long, conflicts_with("html"))]
    live: bool,

    /// Write the plot as embeddable HTML to stdout.
    #[arg(long, conflicts_with("live"))]
    html: bool,

    /// The Prometheus range query.
    ///
    /// Multiple queries can be specified by passing the flag multiple times.
    ///
    /// The number of titles must match the number of queries.
    #[arg(long, short, required(true))]
    query: Vec<String>,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    if let Err(err) = run(opts).await {
        eprintln!("promg encountered an error: {err:#}");
        std::process::exit(1);
    }
}

async fn plot(title: String, items: Vec<QueryResult>) -> eyre::Result<String> {
    // TODO: Histogram/scatter
    let mut plots = Vec::new();
    for result in items.into_iter() {
        // TODO: Custom formatting
        let data = result
            .values
            .into_iter()
            .map(|(x, y)| (UnixTime(x as i64), y.parse::<f64>().unwrap()));
        plots.push(poloto::build::plot(format!("{}", result.metric)).line(data));
    }

    let plot = poloto::header().with_viewbox([1200.0, 800.0]);
    let opt = poloto::render::render_opt()
        .with_tick_lines([true, true])
        .with_viewbox(plot.get_viewbox())
        .move_into();

    let plot = poloto::data(plots)
        .map_opt(|_| opt)
        .build_and_label((title, "Date", ""))
        .append_to(plot.dark_theme());

    plot.render_string().map_err(Into::into)
}

async fn run(mut opts: Opts) -> eyre::Result<()> {
    opts.title.reverse();
    let mut queries: Vec<(Option<String>, RangeQuery)> = Vec::new();
    for query in opts.query.into_iter() {
        queries.push((
            opts.title.pop(),
            RangeQuery {
                query,
                start: opts.start,
                end: opts.end,
                step: opts.step,
            },
        ));
    }

    if opts.html {
        let endpoint = opts.endpoint.as_ref();
        let mut plots: FuturesUnordered<_> = queries
            .into_iter()
            .map(|(title, query)| async move {
                let response = query.send(endpoint).await?;
                plot(title.unwrap_or("".into()), response.data.result).await
            })
            .collect();

        while let Some(plot) = plots.try_next().await? {
            println!("{plot}");
        }
    } else if opts.live {
        let reload_interval = opts.step;
        let live_reload = LiveReloadLayer::new();
        let reloader = live_reload.reloader();
        let app = Router::new()
            .route(
                "/",
                get(move || async move {
                    let endpoint = opts.endpoint.as_ref();
                    let mut plots: FuturesUnordered<_> = queries
                        .clone()
                        .into_iter()
                        .map(|(title, query)| async move {
                            let response = query.send(endpoint).await?;
                            plot(title.unwrap_or("".into()), response.data.result).await
                        })
                        .collect();
                    let mut response = String::new();
                    while let Some(plot) = plots.next().await {
                        match plot {
                            Ok(plot) => response.push_str(&plot),
                            Err(_) => {
                                return (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
                                    .into_response()
                            }
                        }
                    }
                    Html(response).into_response()
                }),
            )
            .layer(live_reload);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(reload_interval / 2)).await;
                reloader.reload();
            }
        });

        let addr: SocketAddr = { TcpListener::bind("127.0.0.1:0").await?.local_addr()? };
        let handle = axum::Server::bind(&addr).serve(app.into_make_service());

        open::that(format!("http://{addr}"))?;
        handle.await?;
    }

    Ok(())
}
