use super::{do_query, MyState};
use rocket::{get, response::status::NotFound, State};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Swap {
    id: String,
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
        let mut swaps = vec![];
        for transaction in self.data.transactions {
            for swap in transaction.swaps {
                swaps.push(swap.id);
            }
        }
        QueryResponse { swaps }
    }
}

/// This struct defines what we want to return to the user making the query. We
/// will be returning an array of Swaps where each Swap is the id of the Pool.
#[derive(Serialize, Deserialize, Debug)]
struct QueryResponse {
    swaps: Vec<String>,
}

/// Handler for looking up the list of swaps in a given block.
#[get("/swaps/<block_number>")]
pub fn query_swaps(block_number: u64, state: &State<MyState>) -> Result<String, NotFound<String>> {
    let the_graph_query = format!("{{\"query\":\"  {{transactions(orderBy: blockNumber, orderDirection: desc, where: {{blockNumber: {}}}) {{swaps {{id}}}}}}\"}}", block_number);
    let the_graph_response_str = match do_query(&state.client, the_graph_query) {
        Ok(input) => input,
        Err(err) => {
            return Err(NotFound(format!(
                "Querying thegraph.com failed with {}",
                err
            )))
        }
    };
    let the_graph_response: TheGraphResponseData =
        match serde_json::from_str(&the_graph_response_str) {
            Ok(response) => response,
            Err(err) => {
                return Err(NotFound(format!(
                    "Converting response from thegraph.com failed with {}",
                    err
                )))
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
