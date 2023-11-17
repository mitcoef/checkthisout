use dotenv::dotenv;
use std::env::var;
use serde_json::to_string;
use std::fmt::Debug;

mod database;

use sea_orm::{Database, DatabaseConnection, DbErr, EntityTrait};
use database::{prelude::*, profiles};

#[tokio::main]
async fn main() -> Result<(), DbErr>{
    dotenv().ok();
    let db_url = var("DATABASE_URL").expect("DATABASE_URL missing from .env");
    let db = Database::connect(&db_url).await.unwrap();

    let results = profiles::Entity::find().all(&db).await.unwrap();

    println!("{}", to_string(&results[0]).unwrap());

    Ok(())

}