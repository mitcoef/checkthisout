use geoutils::Location;

const R: f64 = 6371.0;

pub trait SimpleDistance {
    fn calculate_simple_distance_km(&self, to: &Self) -> f64;
}

impl SimpleDistance for Location {
    fn calculate_simple_distance_km(&self, to: &Self) -> f64 {
        let (lat1, lon1) = (self.latitude().to_radians(), self.longitude().to_radians());
        let (lat2, lon2) = (to.latitude().to_radians(), to.longitude().to_radians());

        (lat1.sin() * lat2.sin() + lat1.cos() * lat2.cos() * (lon1 - lon2).cos()).acos() * R
    }
}
