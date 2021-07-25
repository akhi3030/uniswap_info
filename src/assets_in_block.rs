use super::{do_query, MyState};
use rocket::{get, response::status::NotFound, State};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
struct Token {
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Pair {
    token0: Token,
    token1: Token,
}

#[derive(Serialize, Deserialize, Debug)]
struct Swap {
    pair: Pair,
}

#[derive(Serialize, Deserialize, Debug)]
struct Swaps {
    swaps: Vec<Swap>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TheGraphResponse {
    transactions: Vec<Swaps>,
}

/// This struct along with the the above structs defines the format of the
/// result what we expect to get back from thegraph.com when we make the query
/// defined in `query_swaps()`. Rust does not support nested struct declaration
/// making this definitions a bit messy.
#[derive(Serialize, Deserialize, Debug)]
struct TheGraphResponseData {
    data: TheGraphResponse,
}

impl TheGraphResponseData {
    fn into_query_response(self) -> QueryResponse {
        // Use a hash set to accumulate tokens so that we do not return
        // duplicates.
        let mut tokens = HashSet::new();
        for transaction in self.data.transactions {
            for swap in transaction.swaps {
                tokens.insert(swap.pair.token0.id);
                tokens.insert(swap.pair.token1.id);
            }
        }
        let tokens = tokens.into_iter().collect();
        QueryResponse { tokens }
    }
}

/// This struct defines what we want to return to the user making the query. We
/// will be returning an array of Swaps where each Swap is the id of the Pool.
#[derive(Serialize, Deserialize, Debug)]
struct QueryResponse {
    tokens: Vec<String>,
}

/// Handler for looking up which set of assets are mentioned in all the swaps in
/// all the transactions that were included in a given block.
#[get("/assets_in_block/<block_number>")]
pub fn query_assets_in_block(
    block_number: u64,
    state: &State<MyState>,
) -> Result<String, NotFound<String>> {
    let the_graph_query = format!("{{\"query\":\"{{transactions(orderBy: blockNumber, orderDirection: desc, where: {{blockNumber: {}}}) {{swaps {{pair {{token0 {{id}} token1 {{id}}}}}}}}}}\"}}", block_number);
    let the_graph_response_str = match do_query(&state.client, the_graph_query) {
        Ok(input) => input,
        Err(err) => return Err(NotFound(err)),
    };
    let the_graph_response: TheGraphResponseData =
        match serde_json::from_str(&the_graph_response_str) {
            Ok(response) => response,
            Err(err) => {
                println!("failed: {}", err);
                return Ok(the_graph_response_str);
            }
        };
    let user_response = the_graph_response.into_query_response();
    match serde_json::to_value(user_response) {
        Ok(json) => Ok(json.to_string()),
        Err(err) => Err(NotFound(format!(
            "Converting to user response failed with {}",
            err
        ))),
    }
}
