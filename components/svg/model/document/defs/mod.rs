/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use crate::model::units::Id;

mod clip_path;
mod filter;
mod gradient;
mod marker;
mod mask;
mod pattern;

pub use self::clip_path::ClipPathDef;
pub use self::filter::FilterDef;
pub use self::gradient::GradientDef;
pub use self::marker::MarkerDef;
pub use self::mask::MaskDef;
pub use self::pattern::PatternDef;

pub enum DefRef<T> {
    // Raw '#id' (without '#'), not resolved yet.
    Ref(Id),
    // Typed definition, resolved
    Resolved(Arc<T>),
}

impl<T> Clone for DefRef<T> {
    fn clone(&self) -> Self {
        match self {
            DefRef::Ref(id) => DefRef::Ref(id.clone()),
            DefRef::Resolved(def) => DefRef::Resolved(Arc::clone(def)),
        }
    }
}

impl<T> DefRef<T> {
    pub fn resolved(&self) -> Option<&T> {
        match self {
            DefRef::Resolved(def) => Some(def.as_ref()),
            DefRef::Ref(_) => None,
        }
    }
}
