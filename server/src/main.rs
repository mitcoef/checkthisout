use std::net::SocketAddr;

mod database;
mod rest;
mod utils;

use axum::{
    routing::{get, patch},
    Router,
};
use rest::app_state;
use sea_orm::{DbErr, EntityTrait};
use utils::postcode_utils::Postcode;

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let state = rest::app_state::init_state().await?;

    let router: Router = Router::new()
        .route("/craftsmen", get(rest::get_craftsmen::handler))
        .route("/craftsmen/:id", patch(rest::patch_craftsmen::handler))
        .with_state(state);

    let addr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 1339));
    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
