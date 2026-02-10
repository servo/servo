/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
