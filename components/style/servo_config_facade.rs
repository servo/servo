/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 /* Ref: https://github.com/servo/servo/issues/15460 for more info for why it's needed. */

use servo_config::prefs::PREFS;

// prefs functions


// opts functions
#[cfg(feature = "servo")]
fn style_sharing_stats_enabled() -> bool {
 servo_config::opts::get().style_sharing_stats.unwrap_or(true)
}
#[cfg(feature = "gecko")]
fn style_sharing_stats_enabled() -> bool {true}
