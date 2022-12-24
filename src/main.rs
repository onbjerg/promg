use clap::Parser;
use plotly::{common::Line, layout::Axis, Layout, Plot, Scatter};

mod prometheus;

/// Generate a plot from a Prometheus range query.
#[derive(Parser, Debug)]
struct Opts {
    /// The Prometheus server endpoint.
    #[arg(short, long, default_value = "http://localhost:9090")]
    endpoint: String,

    /// The range query step (in seconds)
    #[arg(short, long, default_value = "60")]
    step: u64,

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
    let response = prometheus::RangeQuery {
        query: opts.query.into_iter().collect(),
        start: 1671820668 - 3600 * 10,
        end: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        step: opts.step,
    }
    .send(&opts.endpoint)
    .await?;

    // TODO: Plot title
    // TODO: Histogram/scatter
    let mut plot = Plot::new();
    let layout = Layout::new()
        .x_axis(Axis::new().auto_range(true))
        .y_axis(Axis::new().auto_range(true));
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
