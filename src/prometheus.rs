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
