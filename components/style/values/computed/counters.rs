/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed values for counter properties

use crate::values::computed::image::Image;
use crate::values::generics::counters as generics;
use crate::values::generics::counters::CounterIncrement as GenericCounterIncrement;
use crate::values::generics::counters::CounterSetOrReset as GenericCounterSetOrReset;

/// A computed value for the `counter-increment` property.
pub type CounterIncrement = GenericCounterIncrement<i32>;

/// A computed value for the `counter-set` and `counter-reset` properties.
pub type CounterSetOrReset = GenericCounterSetOrReset<i32>;

/// A computed value for the `content` property.
pub type Content = generics::GenericContent<Image>;

/// A computed content item.
pub type ContentItem = generics::GenericContentItem<Image>;
