/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo-specific bits of the style system.
//!
//! These get compiled out on a Gecko build.

pub mod restyle_damage;
pub mod selector_parser;
