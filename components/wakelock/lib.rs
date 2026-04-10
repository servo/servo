/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Platform abstraction for the Screen Wake Lock API.
//!
//! Defines [`WakeLockProvider`], a trait for acquiring and releasing OS-level
//! screen wake locks. Platform-specific implementations will be added in
//! follow-up work. For now, [`NoOpWakeLockProvider`] is the only implementation
//! and does nothing.
//!
//! <https://w3c.github.io/screen-wake-lock/>

/// Trait for platform-specific screen wake lock support.
///
/// Implementations are responsible for interacting with the OS to prevent
/// the screen from sleeping while a wake lock is held.
pub trait WakeLockProvider: Send + Sync {
    /// Acquire a screen wake lock, preventing the screen from sleeping.
    /// Called when the aggregate lock count transitions from 0 to 1.
    fn acquire(&self);

    /// Release a previously acquired screen wake lock, allowing the screen
    /// to sleep. Called when the aggregate lock count transitions from N to 0.
    fn release(&self);
}

/// A no-op [`WakeLockProvider`] used when no platform implementation is
/// available. All operations are silently ignored.
pub struct NoOpWakeLockProvider;

impl WakeLockProvider for NoOpWakeLockProvider {
    fn acquire(&self) {}
    fn release(&self) {}
}
