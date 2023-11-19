use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use geoutils::Location;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    database::{filtered_ranks, profiles},
    utils::postcode_utils::{PatchFilters, Postcode},
    utils::ranking::calc_rank,
    utils::scoring,
};

use super::app_state::AppState;
const DEFAULT_DISTANCE: f64 = 80.0;

#[derive(Serialize, Deserialize)]
pub struct ReqBody {
    maxDrivingDistance: Option<f64>,
    profilePictureScore: Option<f64>,
    profileDescriptionScore: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct Updated {
    pub maxDrivingDistance: f64,
    pub profilePictureScore: f64,
    pub profileDescriptionScore: f64,
}

#[derive(Serialize, Deserialize)]
pub struct QueryResult {
    pub id: i32,
    pub updated: Updated,
}

async fn update_score_and_ranks(
    profile: profiles::Model,
    pic_score: Option<f64>,
    desc_score: Option<f64>,
    db: DatabaseConnection,
) -> Result<String, StatusCode> {
    // no max distance was given, at least one score is expected
    let new_score = scoring::calc_score_from_options(
        pic_score,
        desc_score,
        profile.profile_picture_score,
        profile.profile_description_score,
    )
    .ok_or(StatusCode::BAD_REQUEST)?;

    // distance doesn't change, only rank, so query all in preperation for update
    let ranks: Vec<filtered_ranks::ActiveModel> = filtered_ranks::Entity::find()
        .filter(filtered_ranks::Column::ProfileId.eq(profile.id))
        .order_by_asc(filtered_ranks::Column::Distance)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|filter| {
            let dist = filter.distance;
            let mut model: filtered_ranks::ActiveModel = filter.into();
            model.rank = ActiveValue::Set(calc_rank(dist, new_score));
            model
        })
        .collect();

    let mut profile: profiles::ActiveModel = profile.into();

    // update all values that were changed
    if let Some(pic_score) = pic_score {
        profile.profile_picture_score = ActiveValue::Set(pic_score);
    }

    if let Some(desc_score) = desc_score {
        profile.profile_picture_score = ActiveValue::Set(desc_score);
    }

    profile.profile_score = ActiveValue::Set(new_score);

    // perform updates inside of transaction
    let txn = db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    filtered_ranks::Entity::insert_many(ranks)
        .on_empty_do_nothing()
        .on_conflict(
            sea_query::OnConflict::columns([
                filtered_ranks::Column::ProfileId,
                filtered_ranks::Column::Postcode,
            ])
            .update_columns([filtered_ranks::Column::Rank])
            .to_owned(),
        )
        .exec(&txn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let profile = profile
        .update(&txn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    txn.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let query_result: QueryResult = profile.into();

    Ok(serde_json::to_string(&query_result).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}

async fn update_distances(
    profile: profiles::Model,
    pic_score: Option<f64>,
    desc_score: Option<f64>,
    max_driving_distance: f64,
    postcodes: Arc<Vec<Postcode>>,
    db: DatabaseConnection,
) -> Result<String, StatusCode> {
    let id = profile.id;
    let new_score = scoring::calc_score_from_options(
        pic_score,
        desc_score,
        profile.profile_picture_score,
        profile.profile_description_score,
    );

    let patch = PatchFilters {
        profile_id: profile.id,
        // XXX this converts the meters to km, please excuse the magic number
        max_driving_distance: max_driving_distance / 1000.0,
        profile_score: new_score.unwrap_or(profile.profile_score),
        loc: Location::new(profile.lat, profile.lon),
    };

    let mut profile: profiles::ActiveModel = profile.into();

    // update all values that were changed
    if let Some(pic_score) = pic_score {
        profile.profile_picture_score = ActiveValue::Set(pic_score);
    }

    if let Some(desc_score) = desc_score {
        profile.profile_picture_score = ActiveValue::Set(desc_score);
    }

    if let Some(new_score) = new_score {
        profile.profile_score = ActiveValue::Set(new_score);
    }

    profile.max_driving_distance = ActiveValue::Set(max_driving_distance);

    let filters: Vec<filtered_ranks::ActiveModel> = postcodes
        .iter()
        .filter_map(|postcode| postcode.get_model_opt(&patch).map(|model| model.into()))
        .collect();

    let txn = db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // this is a classic Hackathon solution - we should really update existing fields,
    // but it's 2am and we are operating on 3h of sleep...
    filtered_ranks::Entity::delete_many()
        .filter(filtered_ranks::Column::ProfileId.eq(id))
        .exec(&txn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    filtered_ranks::Entity::insert_many(filters)
        .on_empty_do_nothing()
        .on_conflict(
            sea_query::OnConflict::columns([
                filtered_ranks::Column::ProfileId,
                filtered_ranks::Column::Postcode,
            ])
            .update_columns([filtered_ranks::Column::Rank])
            .to_owned(),
        )
        .exec(&txn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let profile = profile
        .update(&txn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    txn.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let query_result: QueryResult = profile.into();

    Ok(serde_json::to_string(&query_result).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}

pub async fn handler(
    Path(id): Path<i32>,
    State(AppState { db, postcodes }): State<AppState>,
    Json(input): Json<ReqBody>,
) -> Result<String, StatusCode> {
    let ReqBody {
        maxDrivingDistance,
        profilePictureScore,
        profileDescriptionScore,
    } = input;

    let profile: profiles::Model = profiles::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    match maxDrivingDistance {
        Some(distance) => {
            return update_distances(
                profile,
                profilePictureScore,
                profileDescriptionScore,
                distance,
                postcodes,
                db,
            )
            .await
        }
        None => {
            return update_score_and_ranks(
                profile,
                profilePictureScore,
                profileDescriptionScore,
                db,
            )
            .await
        }
    }
}
