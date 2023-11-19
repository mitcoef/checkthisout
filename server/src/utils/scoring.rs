pub fn calc_score(pic_score: f64, desc_score: f64) -> f64 {
    0.4 * pic_score + 0.6 * desc_score
}

pub fn calc_score_from_options(
    pic_score: Option<f64>,
    desc_score: Option<f64>,
    pic_score_old: f64,
    desc_score_old: f64,
) -> Option<f64> {
    match (pic_score, desc_score) {
        (None, None) => None,
        (None, Some(desc_score)) => Some(calc_score(pic_score_old, desc_score)),
        (Some(pic_score), None) => Some(calc_score(pic_score, desc_score_old)),
        (Some(pic_score), Some(desc_score)) => Some(calc_score(pic_score, desc_score)),
    }
}
