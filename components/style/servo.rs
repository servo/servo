/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use context;
use data;
use properties::ServoComputedValues;
use selector_impl::ServoSelectorImpl;
use selector_matching;
use stylesheets;

/// Concrete types for servo Style implementation
pub type Stylesheet = stylesheets::Stylesheet<ServoSelectorImpl>;
pub type PrivateStyleData = data::PrivateStyleData<ServoSelectorImpl, ServoComputedValues>;
pub type Stylist = selector_matching::Stylist<ServoSelectorImpl>;
pub type StylistWrapper = context::StylistWrapper<ServoSelectorImpl>;
pub type SharedStyleContext = context::SharedStyleContext<ServoSelectorImpl>;
