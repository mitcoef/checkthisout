use sea_orm::FromQueryResult;

use serde::Serialize;

use crate::database::profiles;
use crate::rest::patch_craftsmen;

#[derive(Serialize)]
pub struct Craftsman {
    id: i32,
    name: String,
    #[serde(rename = "rankingScore")]
    ranking_score: f64,
}

#[derive(FromQueryResult, Serialize)]
pub struct ProfileWithRank {
    id: i32,
    first_name: String,
    last_name: String,
    rank: f64,
}

impl Into<Craftsman> for ProfileWithRank {
    fn into(self) -> Craftsman {
        let Self {
            id,
            first_name,
            last_name,
            rank,
        } = self;

        Craftsman {
            id,
            name: format!("{first_name} {last_name}"),
            ranking_score: rank,
        }
    }
}

impl Into<patch_craftsmen::QueryResult> for profiles::Model {
    fn into(self) -> patch_craftsmen::QueryResult {
        let Self {
            id,
            max_driving_distance,
            profile_picture_score,
            profile_description_score,
            ..
        } = self;

        let updated = patch_craftsmen::Updated {
            max_driving_distance,
            profile_picture_score,
            profile_description_score,
        };

        patch_craftsmen::QueryResult { id, updated }
    }
}
