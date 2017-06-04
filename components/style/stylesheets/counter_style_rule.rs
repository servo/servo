/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// TODO(emilio): unify this, components/style/counter_style.rs, and
// components/style/gecko/rules.rs
#![allow(missing_docs)]

#[cfg(feature = "servo")]
pub use counter_style::CounterStyleRuleData as CounterStyleRule;
#[cfg(feature = "gecko")]
pub use gecko::rules::CounterStyleRule;
