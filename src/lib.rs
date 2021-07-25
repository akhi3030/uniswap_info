mod asset_volume;
mod pools;
mod swaps;

pub use asset_volume::query_asset_volume;
pub use pools::query_pools;
use reqwest::Client as ReqwestClient;
pub use swaps::query_swaps;

pub struct MyState {
    client: ReqwestClient,
}

impl MyState {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }
}

/// Given the body of a POST query request, queries the thegraph.com's
/// uniswap-v2 subgraph.
fn do_query(client: &ReqwestClient, body: String) -> Result<String, String> {
    let url = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v2";
    let request_builder = client.post(url).body(body);
    let mut response = match request_builder.send() {
        Ok(response) => response,
        Err(err) => return Err(err.to_string()),
    };
    match response.text() {
        Ok(text) => Ok(text),
        Err(err) => Err(err.to_string()),
    }
}
