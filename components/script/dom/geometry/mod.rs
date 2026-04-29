/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod dommatrix;
pub(crate) mod dommatrixreadonly;
pub(crate) mod dompoint;
pub(crate) mod dompointreadonly;
pub(crate) mod domquad;
pub(crate) mod domrect;
pub(crate) mod domrectlist;
pub(crate) mod domrectreadonly;

// Re-export geometry interfaces so they remain accessible via crate::dom::*
pub(crate) use dommatrix::*;
pub(crate) use dommatrixreadonly::*;
pub(crate) use dompoint::*;
pub(crate) use dompointreadonly::*;
pub(crate) use domquad::*;
pub(crate) use domrect::*;
pub(crate) use domrectlist::*;
pub(crate) use domrectreadonly::*;