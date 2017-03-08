/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 /* Ref: https://github.com/servo/servo/issues/15460 for more info for why it's needed. */

// TODO: write docs for this mod
use servo_config;

// prefs functions
#[cfg(feature = "servo")] #[allow(missing_docs)]
pub fn layout_viewport_enabled() -> bool {
    servo_config::prefs::PREFS.get("layout.viewport.enabled").as_boolean().unwrap_or(false)
}
#[cfg(feature = "gecko")] #[allow(missing_docs)]
pub fn layout_viewport_enabled() -> bool { false }

// opts functions
#[cfg(feature = "servo")] #[allow(missing_docs)]
pub fn style_sharing_stats_enabled() -> bool {
    servo_config::opts::get().style_sharing_stats
}
#[cfg(feature = "gecko")] #[allow(missing_docs)]
pub fn style_sharing_stats_enabled() -> bool { true }

#[cfg(feature = "servo")] #[allow(missing_docs)]
pub fn disable_share_style_cache_enabled() -> bool {
    servo_config::opts::get().disable_share_style_cache
}
#[cfg(feature = "gecko")] #[allow(missing_docs)]
pub fn disable_share_style_cache_enabled() -> bool { true }

#[cfg(feature = "servo")] #[allow(missing_docs)]
pub fn nonincremental_layout_enabled() -> bool {
    servo_config::opts::get().nonincremental_layout
}
#[cfg(feature = "gecko")] #[allow(missing_docs)]
pub fn nonincremental_layout_enabled() -> bool { false }
