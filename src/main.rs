use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env::var;
use std::fmt::Debug;
use std::net::SocketAddr;

mod database;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use database::{filtered_ranks, prelude::*, profiles};
use sea_orm::{ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

#[derive(Serialize, Deserialize, Debug)]
pub struct ReqQuery {
    postalcode: String,
}

pub async fn handler(
    Query(ReqQuery { postalcode }): Query<ReqQuery>,
) -> Result<String, StatusCode> {
    dotenv().ok();
    let db_url = var("DATABASE_URL").expect("DATABASE_URL missing from .env");
    let db = Database::connect(&db_url).await.unwrap();

    let postcode = postalcode
        .parse::<i32>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let result = filtered_ranks::Entity::find()
        .filter(filtered_ranks::Column::Postcode.eq(postcode))
        .all(&db)
        .await
        .unwrap();

    let mut profiles : Vec<profiles::Model> = Vec::new();

    for model in result {
        let profile = profiles::Entity::find_by_id(model.profile_id).one(&db).await.unwrap().unwrap();
        profiles.push(profile);
    }

    Ok(serde_json::to_string(&profiles).unwrap())
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    // dotenv().ok();
    // let db_url = var("DATABASE_URL").expect("DATABASE_URL missing from .env");
    // let db = Database::connect(&db_url).await.unwrap();

    let router: Router = Router::new().route("/craftsmen", get(handler));

    let addr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 1339));
    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
