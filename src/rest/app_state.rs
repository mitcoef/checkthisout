use dotenv::dotenv;
use sea_orm::{Database, EntityTrait};
use sea_orm::{DatabaseConnection, DbErr};
use std::env::var;
use std::rc::Rc;
use std::sync::Arc;

use crate::database::{self, postcode};
use crate::utils::postcode_utils::Postcode;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub postcodes: Arc<Vec<Postcode>>,
}

pub async fn init_state() -> Result<AppState, DbErr> {
    dotenv().ok();
    let db_url = var("DATABASE_URL").expect("DATABASE_URL missing from .env");
    let db = Database::connect(&db_url).await?;

    let postcodes: Vec<Postcode> = database::postcode::Entity::find()
        .all(&db)
        .await
        .unwrap()
        .into_iter()
        .map(|postcode| postcode.into())
        .collect();

    let postcodes = Arc::new(postcodes);

    Ok(AppState { db, postcodes })
}
