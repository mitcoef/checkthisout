use crate::database::profiles;
use crate::rest::patch_craftsmen;

pub struct Craftsman {
    id: i32,
    name: String,
    rankingScore: f64,
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
            maxDrivingDistance: max_driving_distance,
            profilePictureScore: profile_picture_score,
            profileDescriptionScore: profile_description_score,
        };

        patch_craftsmen::QueryResult { id, updated }
    }
}
