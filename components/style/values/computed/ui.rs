/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed values for UI properties

use crate::values::computed::color::Color;
use crate::values::computed::url::ComputedImageUrl;
use crate::values::computed::Number;
use crate::values::generics::ui as generics;

pub use crate::values::specified::ui::CursorKind;
pub use crate::values::specified::ui::{MozForceBrokenImageIcon, UserSelect};

/// A computed value for the `cursor` property.
pub type Cursor = generics::Cursor<CursorImage>;

/// A computed value for item of `image cursors`.
pub type CursorImage = generics::CursorImage<ComputedImageUrl, Number>;

/// A computed value for `scrollbar-color` property.
pub type ScrollbarColor = generics::GenericScrollbarColor<Color>;
