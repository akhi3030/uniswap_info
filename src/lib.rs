mod asset_volume;
mod pools;

pub use asset_volume::query_asset_volume;
pub use pools::query_pools;
use reqwest::Client as ReqwestClient;
use rocket::{get, response::status::NotFound, State};

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

#[get("/block/<block_number>")]
pub fn explore_block(
    block_number: u128,
    state: &State<MyState>,
) -> Result<String, NotFound<String>> {
    // The following gives the list of transactions that were included in the
    // provided block. Now that we have a list of transactions, we can build a
    // query to get a list of swaps in each transaction in the list. Once we
    // have a list of swaps, we can then build another query to get the set of
    // assets that were swaped in each swap in the list.
    let body = format!(
        "{{transactions(orderBy: {}, orderDirection: desc, where: {{blockNumber: 1234}}) {{id}}}}",
        block_number
    );
    Ok(do_query(&state.client, body).unwrap())
}
