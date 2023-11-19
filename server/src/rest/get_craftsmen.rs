use crate::{
    database::{filtered_ranks, profiles},
    utils::profile::Craftsman,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use sea_orm::{
    ColumnTrait, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use serde::{Deserialize, Serialize};

use crate::utils::profile;

use super::app_state::AppState;

const LIMIT: u64 = 20;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReqQuery {
    postalcode: String,
    offset: Option<u64>,
}

#[derive(Serialize)]
pub struct Response {
    craftsmen: Vec<Craftsman>,
}

pub async fn handler(
    Query(ReqQuery { postalcode, offset }): Query<ReqQuery>,
    State(AppState { db, .. }): State<AppState>,
) -> Result<String, StatusCode> {
    let postcode = postalcode
        .parse::<i32>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // TODO make the filter a subquery and then join with that (see if that does us any good)
    let craftsmen: Vec<Craftsman> = profiles::Entity::find()
        .column_as(filtered_ranks::Column::Rank, "rank")
        .join(JoinType::LeftJoin, profiles::Relation::FilteredRanks.def())
        .filter(filtered_ranks::Column::Postcode.eq(postcode))
        .order_by_desc(filtered_ranks::Column::Rank)
        .offset(offset)
        .limit(Some(LIMIT))
        .into_model::<profile::ProfileWithRank>()
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|profile| profile.into())
        .collect();

    Ok(serde_json::to_string(&Response { craftsmen })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}
