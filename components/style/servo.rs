/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
//! Concrete types for servo Style implementation

use animation;
use context;
use data;
use selector_matching;
use stylesheets;

pub type Stylesheet = stylesheets::Stylesheet;
pub type PrivateStyleData = data::PrivateStyleData;
pub type Stylist = selector_matching::Stylist;
pub type SharedStyleContext = context::SharedStyleContext;
pub type LocalStyleContextCreationInfo = context::LocalStyleContextCreationInfo;
pub type Animation = animation::Animation;
