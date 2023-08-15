/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! [Media queries][mq].
//!
//! [mq]: https://drafts.csswg.org/mediaqueries/

mod media_list;
mod media_query;

pub use self::media_list::MediaList;
pub use self::media_query::{MediaQuery, MediaQueryType, MediaType, Qualifier};

#[cfg(feature = "gecko")]
pub use crate::gecko::media_queries::Device;
#[cfg(feature = "servo")]
pub use crate::servo::media_queries::Device;
