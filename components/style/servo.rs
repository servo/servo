/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
//! Concrete types for servo Style implementation

use animation;
use context;
use data;
use selector_matching;
use servo_selector_impl::ServoSelectorImpl;
use stylesheets;

pub type Stylesheet = stylesheets::Stylesheet<ServoSelectorImpl>;
pub type PrivateStyleData = data::PrivateStyleData<ServoSelectorImpl>;
pub type Stylist = selector_matching::Stylist<ServoSelectorImpl>;
pub type SharedStyleContext = context::SharedStyleContext<ServoSelectorImpl>;
pub type LocalStyleContextCreationInfo = context::LocalStyleContextCreationInfo;
pub type Animation = animation::Animation;
