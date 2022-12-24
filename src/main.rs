use clap::Parser;
use plotly::{common::Line, layout::Axis, Layout, Plot, Scatter};
use prometheus::QueryResultType;

mod prometheus;

async fn query(
    upstream: &str,
    q: String,
    start: u64,
    end: u64,
    step: u64,
) -> Result<prometheus::Response, reqwest::Error> {
    let params = [
        ("query", q),
        ("start", start.to_string()),
        ("end", end.to_string()),
        ("step", step.to_string()),
    ];
    let response: prometheus::Response = reqwest::Client::new()
        .post(format!("{upstream}/api/v1/query_range"))
        .form(&params)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if response.data.result_type != QueryResultType::Matrix {
        unimplemented!()
    }

    Ok(response)
}

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
    let response = query(
        &opts.endpoint,
        opts.query.into_iter().collect(),
        1671820668 - 3600 * 10,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        opts.step,
    )
    .await
    .expect("Request failed");

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
}
