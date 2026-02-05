pub struct ConstrainRange<T> {
    pub min: Option<T>,
    pub max: Option<T>,
    pub ideal: Option<T>,
}

pub enum ConstrainBool {
    Ideal(bool),
    Exact(bool),
}

#[derive(Default)]
pub struct MediaTrackConstraintSet {
    pub width: Option<Constrain<u32>>,
    pub height: Option<Constrain<u32>>,
    pub aspect: Option<Constrain<f64>>,
    pub frame_rate: Option<Constrain<f64>>,
    pub sample_rate: Option<Constrain<u32>>,
}

pub enum Constrain<T> {
    Value(T),
    Range(ConstrainRange<T>),
}
