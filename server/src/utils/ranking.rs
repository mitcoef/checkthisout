const DEFAULT_DISTANCE: f64 = 80.0;

pub fn calc_rank(dist: f64, score: f64) -> f64 {
    let distance_score = 1.0 - (dist / DEFAULT_DISTANCE);

    let distance_weight = if dist > DEFAULT_DISTANCE { 0.01 } else { 0.15 };

    distance_weight * distance_score + (1.0 - distance_weight) * score
}
