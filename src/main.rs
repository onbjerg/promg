use crate::prometheus::QueryResult;
use axum::{response::Html, routing::get, Router};
use clap::Parser;
use poloto::num::timestamp::UnixTime;
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
    #[arg(short, long)]
    title: Option<String>,

    /// Open the plot in the browser and live-reload it.
    #[arg(long, conflicts_with("html"))]
    live: bool,

    /// Write the plot as embeddable HTML to stdout.
    #[arg(long, conflicts_with("live"))]
    html: bool,

    /// The Prometheus range query.
    #[arg(required(true))]
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
    let query: String = opts.query.into_iter().collect();
    let title = opts.title.take().unwrap_or_else(|| query.clone());
    let query = prometheus::RangeQuery {
        query: query.clone(),
        start: opts.start,
        end: opts.end,
        step: opts.step,
    };

    if opts.html {
        let response = query.send(&opts.endpoint).await?;
        println!("{}", plot(title, response.data.result).await?);
    } else if opts.live {
        let reload_interval = opts.step;
        let live_reload = LiveReloadLayer::new();
        let reloader = live_reload.reloader();
        let app = Router::new()
            .route(
                "/",
                get(move || async move {
                    Html(
                        plot(
                            title.clone(),
                            query
                                .clone()
                                .send(&opts.endpoint.clone())
                                .await
                                .unwrap()
                                .data
                                .result,
                        )
                        .await
                        .unwrap(),
                    )
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
