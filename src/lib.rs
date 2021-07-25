mod asset_volume;
mod assets_in_block;
mod pools;
mod swaps;

pub use asset_volume::query_asset_volume;
pub use assets_in_block::query_assets_in_block;
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
    let request_builder = client.post(url).body(body.clone());
    let mut response = match request_builder.send() {
        Ok(response) => response,
        Err(err) => {
            return Err(format!(
                "Query {} against thegraph.com failed with {}",
                body, err
            ))
        }
    };
    match response.text() {
        Ok(text) => Ok(text),
        Err(err) => Err(format!(
            "Query {} against thegraph.com: extracting text failed with {}",
            body, err
        )),
    }
}
