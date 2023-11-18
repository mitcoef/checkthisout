// use crate::database::{filtered_ranks, prelude::*, profiles};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
// use sea_orm::{
//     ColumnTrait, Database, DatabaseConnection, DbBackend, DbErr, EntityTrait, JoinType,
//     QueryFilter, QueryOrder, QuerySelect, QueryTrait, RelationTrait,
// };
use serde::{Deserialize, Serialize};

use super::app_state::AppState;

#[derive(Serialize, Deserialize)]
pub struct ReqBody {
    maxDrivingDistance: Option<f64>,
    profilePictureScore: Option<f64>,
    profileDescriptionScore: Option<f64>,
}

pub async fn handler(
    Path(id): Path<i32>,
    State(AppState { db, postcodes }): State<AppState>,
    Json(input): Json<ReqBody>,
) -> Result<String, StatusCode> {
    Ok(serde_json::to_string(&input).unwrap())
}
