/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Invalidation of element styles due to attribute or style changes.

pub mod element_wrapper;
pub mod invalidation_map;
pub mod invalidator;
pub mod restyle_hints;
pub mod state_and_attributes;
