use geoutils::Location;

use crate::database::sea_orm_active_enums::InGroup;
use crate::database::{filtered_ranks, postcode};
use crate::traits::simple_disctance::SimpleDistance;

use super::ranking::calc_rank;

pub struct Postcode {
    postcode: i32,
    loc: Location,
    offset: f64,
}

impl InGroup {
    pub fn get_offset_km(&self) -> f64 {
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
            offset: postcode_extension_distance_group.get_offset_km(),
        }
    }
}

// TODO renameme
pub struct PatchFilters {
    pub profile_id: i32,
    pub max_driving_distance: f64,
    pub profile_score: f64,
    pub loc: Location,
}

impl Postcode {
    pub fn get_model_opt(&self, patch: &PatchFilters) -> Option<filtered_ranks::Model> {
        let Self {
            postcode,
            loc,
            offset,
        } = self;

        let dist: f64 = loc.calculate_simple_distance_km(&patch.loc);

        if dist > patch.max_driving_distance + offset {
            return None;
        }

        let rank = calc_rank(dist, patch.profile_score);

        Some(filtered_ranks::Model {
            profile_id: patch.profile_id,
            postcode: *postcode,
            distance: dist,
            rank: rank,
        })
    }
}
