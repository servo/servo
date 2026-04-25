/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Platform abstraction for the Screen Wake Lock API.
//!
//! Defines [`WakeLockDelegate`], a trait for acquiring and releasing OS-level
//! wake locks. Platform-specific implementations will be added in follow-up
//! work. For now, [`DefaultWakeLockDelegate`] is the only implementation and
//! does nothing.
//!
//! <https://w3c.github.io/screen-wake-lock/>
use std::error::Error;

use embedder_traits::{WakeLockDelegate, WakeLockType};

/// A no-op [`WakeLockDelegate`] used when no platform implementation is
/// available. All operations succeed silently.
pub struct DefaultWakeLockDelegate;

impl WakeLockDelegate for DefaultWakeLockDelegate {
    fn acquire(&self, _type_: WakeLockType) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn release(&self, _type_: WakeLockType) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
