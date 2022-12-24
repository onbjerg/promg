use clap::Parser;
use plotly::{common::Line, layout::Axis, Layout, Plot, Scatter};
use prometheus::QueryResultType;

mod prometheus {
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Debug, Eq, PartialEq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum Status {
        Success,
        Error,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        pub status: Status,
        pub data: Data,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Data {
        pub result_type: QueryResultType,
        pub result: Vec<QueryResult>,
    }

    #[derive(Debug, Eq, PartialEq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum QueryResultType {
        Matrix,
        Vector,
        Scalar,
        String,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryResult {
        pub metric: Metric,
        // TODO: Strengthen this
        pub values: Vec<(f64, String)>,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Metric {
        #[serde(rename = "__name__")]
        pub name: String,
        #[serde(flatten)]
        pub labels: HashMap<String, String>,
    }
}

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

#[derive(Parser, Debug)]
struct Opts {
    #[arg(short, long, default_value = "http://localhost:9090")]
    endpoint: String,

    #[arg(short, long, default_value = "60")]
    step: u64,

    #[arg(long, default_value = "true")]
    open: bool,

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
            .name(result.metric.name)
            .line(Line::new().dash(plotly::common::DashType::Solid));
        plot.add_trace(trace);
    }

    if opts.open {
        plot.show();
    }
}
