/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub struct Opacity(f32);
impl Opacity {
    pub const ONE: Opacity = Opacity(1.0);

    pub fn new(value: f32) -> Opacity {
        Opacity(value.clamp(0.0, 1.0))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for Opacity {
    fn default() -> Self {
        Opacity::ONE
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(String);

impl Id {
    pub fn new(id: impl Into<String>) -> Id {
        Id(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub struct Length(f32);

impl Length {
    pub const ZERO: Length = Length(0.0);

    pub fn new(value: f32) -> Length {
        Length(value)
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for Length {
    fn default() -> Self {
        Length::ZERO
    }
}
