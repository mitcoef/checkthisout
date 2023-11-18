use crate::database::sea_orm_active_enums::InGroup;
use crate::database::{filtered_ranks, postcode};
use crate::rest::patch_craftsmen;

pub struct Postcode {
    postcode: i32,
    lat: f64,
    lon: f64,
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
            lat,
            lon,
            offset: postcode_extension_distance_group.get_offset(),
        }
    }
}

// TODO renameme
pub struct PatchFilters {
    max_driving_distance: i64,
    profile_score: i64,
}

impl Postcode {
    pub fn get_model_opt(&self, patch: PatchFilters) -> Option<filtered_ranks::ActiveModel> {
        let Self {
            postcode,
            lat,
            lon,
            offset,
        } = self;

        todo!()
    }
}
