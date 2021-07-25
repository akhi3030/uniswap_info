use super::{do_query, MyState};
use rocket::{get, response::status::NotFound, State};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct TheGraphResponse {
    t0: Vec<Pool>,
    t1: Vec<Pool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Pool {
    id: String,
}

/// This struct along with the the above structs defines the format of the
/// result what we expect to get back from thegraph.com when we make the query
/// defined in `query_pools()`. Rust does not support nested struct declaration
/// making this definitions a bit messy.
#[derive(Serialize, Deserialize, Debug)]
struct TheGraphResponseData {
    data: TheGraphResponse,
}

impl TheGraphResponseData {
    /// Convert the result from thegraph.com to the result that we want to
    /// return to the user making the query.
    fn into_query_response(mut self) -> QueryResponse {
        // We get two arrays back from thegraph.com, combine and return them as
        // a single array.
        self.data.t0.extend(self.data.t1);
        QueryResponse {
            pools: self.data.t0.into_iter().map(|token| token.id).collect(),
        }
    }
}

/// This struct defines what we want to return to the user making the query. We
/// will be returning an array of Pools where each Pool is the id of the Pool.
#[derive(Serialize, Deserialize, Debug)]
struct QueryResponse {
    pools: Vec<String>,
}

/// Handler to looking up which pools a given token belongs to.
#[get("/pools/<token>")]
pub fn query_pools(token: String, state: &State<MyState>) -> Result<String, NotFound<String>> {
    // To find all the pools for the given `token`, look for pools where the
    // `token` is either token0 or token1.
    //
    // TODO: This is effectively doing 2 queries against thegraph.com. I don't
    // have to worry about the atomicity of the two queries but parsing the
    // result of the two queries is a bit more complicated. There must be a
    // cleaner way of doing this query.
    //
    // Note to reviewer: I would love to hear if you have thoughts on how I
    // could improve this query.
    let the_graph_query = format!("{{\"query\":\"{{t0:pairs(orderDirection:desc, where: {{token0: \\\"{}\\\"}}){{id}}t1:pairs(orderDirection:desc, where: {{token0: \\\"{}\\\"}}){{id}}}}\"}}", token, token);
    let the_graph_response_str = match do_query(&state.client, the_graph_query) {
        Ok(input) => input,
        Err(err) => {
            return Err(NotFound(format!(
                "Querying thegraph.com failed with {}",
                err
            )))
        }
    };

    let the_query_response: TheGraphResponseData =
        match serde_json::from_str(&the_graph_response_str) {
            Ok(response) => response,
            Err(err) => {
                return Err(NotFound(format!(
                    "Converting response from thegraph.com failed with {}",
                    err
                )))
            }
        };
    let user_response = the_query_response.into_query_response();
    match serde_json::to_value(user_response) {
        Ok(json) => Ok(json.to_string()),
        Err(err) => Err(NotFound(format!(
            "Converting to user response failed with {}",
            err
        ))),
    }
}
