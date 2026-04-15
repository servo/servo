/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Platform abstraction for the Screen Wake Lock API.
//!
//! Defines [`WakeLockProvider`], a trait for acquiring and releasing OS-level
//! wake locks. Platform-specific implementations will be added in follow-up
//! work. For now, [`NoOpWakeLockProvider`] is the only implementation and
//! does nothing.
//!
//! <https://w3c.github.io/screen-wake-lock/>
use std::error::Error;

use serde::{Deserialize, Serialize};

/// The type of wake lock to acquire or release.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WakeLockType {
    Screen,
}

/// Trait for platform-specific wake lock support.
///
/// Implementations are responsible for interacting with the OS to prevent
/// the screen (or other resources) from sleeping while a wake lock is held.
pub trait WakeLockProvider: Send + Sync {
    /// Acquire a wake lock of the given type, preventing the associated
    /// resource from sleeping. Called when the aggregate lock count transitions
    /// from 0 to 1. Returns an error if the OS fails to grant the lock.
    fn acquire(&self, type_: WakeLockType) -> Result<(), Box<dyn Error>>;

    /// Release a previously acquired wake lock of the given type, allowing
    /// the resource to sleep. Called when the aggregate lock count transitions
    /// from N to 0.
    fn release(&self, type_: WakeLockType) -> Result<(), Box<dyn Error>>;
}

/// A no-op [`WakeLockProvider`] used when no platform implementation is
/// available. All operations succeed silently.
pub struct NoOpWakeLockProvider;

impl WakeLockProvider for NoOpWakeLockProvider {
    fn acquire(&self, _type_: WakeLockType) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn release(&self, _type_: WakeLockType) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
