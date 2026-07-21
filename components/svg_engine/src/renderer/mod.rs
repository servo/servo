/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod circle;
pub(crate) mod ellipse;
pub(crate) mod line;
pub(crate) mod path;
pub(crate) mod polygon;
pub(crate) mod polyline;
pub(crate) mod rect;
pub(crate) mod render_trait;

pub(crate) use render_trait::Render;
