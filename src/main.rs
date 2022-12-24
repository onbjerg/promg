use clap::Parser;
use plotly::{common::Line, layout::Axis, Layout, Plot, Scatter};

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
#[derive(Parser, Debug)]
struct Opts {
    /// The Prometheus server endpoint.
    #[arg(short, long, default_value = "http://localhost:9090")]
    endpoint: String,

    /// The start of the query range (UNIX timestamp)
    ///
    /// Defaults to 24 hours ago.
    #[arg(short, long, default_value_t = yesterday())]
    start: u64,

    /// The end of the query range (UNIX timestamp)
    ///
    /// Defaults to now.
    #[arg(short, long, default_value_t = now())]
    end: u64,

    /// The range query step (in seconds)
    #[arg(short, long, default_value_t = 60)]
    step: u64,

    /// The title of the plot.
    #[arg(short, long)]
    title: Option<String>,

    /// Open the plot in the browser.
    #[arg(long)]
    open: bool,

    /// Write the plot as embeddable HTML to stdout.
    #[arg(long, default_value = "true")]
    html: bool,

    /// The Prometheus range query.
    #[arg(required(true))]
    query: Vec<String>,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    if let Err(err) = run(opts).await {
        eprintln!("promg encountered an error: {err}");
        std::process::exit(1);
    }
}

async fn run(opts: Opts) -> eyre::Result<()> {
    let query: String = opts.query.into_iter().collect();
    let title = opts.title.as_ref().unwrap_or(&query);
    let response = prometheus::RangeQuery {
        query: query.clone(),
        start: opts.start,
        end: opts.end,
        step: opts.step,
    }
    .send(&opts.endpoint)
    .await?;

    // TODO: Histogram/scatter
    let mut plot = Plot::new();
    let layout = Layout::new()
        .x_axis(Axis::new().auto_range(true))
        .y_axis(Axis::new().auto_range(true))
        .title(<String as AsRef<str>>::as_ref(title).into());
    plot.set_layout(layout);

    for result in response.data.result.into_iter() {
        // TODO: Custom formatting
        let (x, y): (Vec<f64>, Vec<String>) = result.values.into_iter().unzip();
        let trace = Scatter::new(x, y)
            .mode(plotly::common::Mode::Lines)
            .name(result.metric.to_string())
            .line(Line::new().dash(plotly::common::DashType::Solid));
        plot.add_trace(trace);
    }

    if opts.open {
        plot.show();
    }

    if opts.html {
        println!("{}", plot.to_inline_html(None));
    }

    Ok(())
}
