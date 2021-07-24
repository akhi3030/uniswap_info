use super::{do_query, MyState};
use rocket::{get, response::status::NotFound, State};
use serde::{Deserialize, Serialize};

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
    // Rust best practices say that variable names should be snake case so use
    // serde's rename field attribute to prescribe the actual name of the field.
    #[serde(rename = "amountUSD")]
    amount_usd: String,
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
/// defined in `query_asset_volume()`. Rust does not support nested struct
/// declaration making this definitions a bit messy.
#[derive(Serialize, Deserialize, Debug)]
struct TheGraphResponseData {
    data: TheGraphResponse,
}

impl TheGraphResponseData {
    /// Convert the result from thegraph.com to the USD amount to return to the
    /// user. First we filter the result from the thegraph.com to only look at
    /// the swaps that contain the token of interest and then we add the usd
    /// amount
    fn usd_amount(self, token: String) -> f64 {
        let mut total = 0.0;
        for transaction in self.data.transactions {
            for swap in transaction.swaps {
                if swap.pair.token0.id == token || swap.pair.token1.id == token {
                    // If we fail to convert the string to float, we could
                    // return an error or just ignore the value.
                    let amount = swap.amount_usd.parse::<f64>().unwrap_or_else(|_| 0.0);
                    total += amount;
                }
            }
        }
        total
    }
}

#[get("/asset_volume/<token>/<start>/<end>")]
pub fn query_asset_volume(
    token: String,
    start: u64,
    end: u64,
    state: &State<MyState>,
) -> Result<String, NotFound<String>> {
    let the_graph_query = format!("{{\"query\":\"{{transactions(where: {{timestamp_gt: {}, timestamp_lt: {}}}) {{swaps {{amountUSD pair {{token0 {{id}}token1{{id}}}}}}}}}}\"}}", start, end);
    let the_graph_response_str = match do_query(&state.client, the_graph_query) {
        Ok(response) => response,
        Err(err) => return Err(NotFound(err)),
    };
    let the_graph_response: TheGraphResponseData =
        match serde_json::from_str(&the_graph_response_str) {
            Ok(response) => response,
            Err(err) => return Err(NotFound(err.to_string())),
        };
    Ok(the_graph_response.usd_amount(token).to_string())
}
