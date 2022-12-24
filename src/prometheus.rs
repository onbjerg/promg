use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

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

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;

        let labels: Vec<(&String, &String)> = self
            .labels
            .iter()
            .filter(|(key, _)| key != &"job")
            .collect();

        if !labels.is_empty() {
            write!(f, "{{")?;
            for (key, value) in labels {
                write!(f, "{key}={value}")?;
            }
            write!(f, "}}")?;
        }

        Ok(())
    }
}

pub async fn range_query(
    upstream: &str,
    q: String,
    start: u64,
    end: u64,
    step: u64,
) -> Result<Response, reqwest::Error> {
    let params = [
        ("query", q),
        ("start", start.to_string()),
        ("end", end.to_string()),
        ("step", step.to_string()),
    ];
    let response: Response = reqwest::Client::new()
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
