/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Code shared between [media queries][mq] and [container queries][cq].
//!
//! [mq]: https://drafts.csswg.org/mediaqueries/
//! [cq]: https://drafts.csswg.org/css-contain-3/#container-rule

pub mod condition;

#[macro_use]
pub mod feature;
pub mod feature_expression;
pub mod values;

pub use self::condition::QueryCondition;
pub use self::feature::FeatureFlags;
pub use self::feature_expression::{FeatureType, QueryFeatureExpression};
