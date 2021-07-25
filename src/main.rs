use reqwest::ClientBuilder as ReqwestClientBuilder;
use rocket::{launch, routes};
use uniswap_info_lib::{query_asset_volume, query_pools, query_swaps, MyState};

#[launch]
fn rocket() -> _ {
    // Create a Reqwest client upfront for all subsequent queries to
    // thegraph.com so that we do not have to keep initialising and loading the
    // TLS backend. If something goes wrong, fail.
    //
    // TODO: return a better error message to user.
    let client = ReqwestClientBuilder::new().build().unwrap();

    // Starts a http server listening at localhost:8000.
    //
    // TODO: let user control the IP:port the server is listening on.
    rocket::build()
        .mount("/", routes![query_pools, query_asset_volume, query_swaps])
        .manage(MyState::new(client))
}
