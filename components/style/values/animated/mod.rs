/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated values.
//!
//! Some values, notably colors, cannot be interpolated directly with their
//! computed values and need yet another intermediate representation. This
//! module's raison d'être is to ultimately contain all these types.

pub mod effects;
