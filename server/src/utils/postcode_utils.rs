use axum::routing::post;
use geoutils::Location;
use sea_orm::ActiveValue;

use crate::database::sea_orm_active_enums::InGroup;
use crate::database::{filtered_ranks, postcode};
use crate::rest::patch_craftsmen;

const DEFAULT_DISTANCE: f64 = 80.0;

pub struct Postcode {
    postcode: i32,
    loc: Location,
    offset: f64,
}

impl InGroup {
    pub fn get_offset(&self) -> f64 {
        match self {
            InGroup::GroupA => 0.0,
            InGroup::GroupB => 2.0,
            InGroup::GroupC => 5.0,
        }
    }
}

impl Into<Postcode> for postcode::Model {
    fn into(self) -> Postcode {
        let Self {
            postcode,
            lat,
            lon,
            postcode_extension_distance_group,
            ..
        } = self;

        Postcode {
            postcode,
            loc: Location::new(lat, lon),
            offset: postcode_extension_distance_group.get_offset(),
        }
    }
}

// TODO renameme
pub struct PatchFilters {
    profile_id: i32,
    max_driving_distance: f64,
    profile_score: f64,
    loc: Location,
}

impl Postcode {
    pub fn get_model_opt(&self, patch: &PatchFilters) -> Option<filtered_ranks::ActiveModel> {
        let Self {
            postcode,
            loc,
            offset,
        } = self;

        let dist: f64 = loc.haversine_distance_to(&patch.loc).meters();

        if dist > patch.max_driving_distance + offset {
            return None;
        }

        let distance_score = 1.0 - dist / DEFAULT_DISTANCE;

        let distance_weight = if dist > DEFAULT_DISTANCE { 0.01 } else { 0.15 };

        let rank = distance_weight * distance_score + (1.0 - distance_weight) * patch.profile_score;

        Some(filtered_ranks::ActiveModel {
            profile_id: ActiveValue::Set(patch.profile_id),
            postcode: ActiveValue::Set(*postcode),
            distance: ActiveValue::Set(dist),
            rank: ActiveValue::Set(rank),
        })
    }
}
