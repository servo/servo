/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [Media queries][mq].
//!
//! [mq]: https://drafts.csswg.org/mediaqueries/

mod media_condition;
mod media_list;
mod media_query;
#[macro_use]
pub mod media_feature;
pub mod media_feature_expression;

pub use self::media_condition::MediaCondition;
pub use self::media_list::MediaList;
pub use self::media_query::{MediaQuery, MediaQueryType, MediaType, Qualifier};
pub use self::media_feature_expression::MediaFeatureExpression;

#[cfg(feature = "servo")]
pub use servo::media_queries::Device;
#[cfg(feature = "gecko")]
pub use gecko::media_queries::Device;
