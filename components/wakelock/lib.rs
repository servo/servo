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

/// The type of wake lock to acquire or release.
///
/// Currently only `Screen` is defined by the spec. Additional variants
/// (e.g. `Cpu`) may be added in the future.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WakeLockType {
    Screen,
}

/// Error returned when acquiring a wake lock fails at the OS level.
#[derive(Debug)]
pub struct WakeLockError(pub String);

impl std::fmt::Display for WakeLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wake lock error: {}", self.0)
    }
}

impl std::error::Error for WakeLockError {}

/// Trait for platform-specific wake lock support.
///
/// Implementations are responsible for interacting with the OS to prevent
/// the screen (or other resources) from sleeping while a wake lock is held.
pub trait WakeLockProvider: Send + Sync {
    /// Acquire a wake lock of the given type, preventing the associated
    /// resource from sleeping. Called when the aggregate lock count transitions
    /// from 0 to 1. Returns an error if the OS fails to grant the lock.
    fn acquire(&self, type_: WakeLockType) -> Result<(), WakeLockError>;

    /// Release a previously acquired wake lock of the given type, allowing
    /// the resource to sleep. Called when the aggregate lock count transitions
    /// from N to 0.
    fn release(&self, type_: WakeLockType);
}

/// A no-op [`WakeLockProvider`] used when no platform implementation is
/// available. All operations succeed silently.
pub struct NoOpWakeLockProvider;

impl WakeLockProvider for NoOpWakeLockProvider {
    fn acquire(&self, _type_: WakeLockType) -> Result<(), WakeLockError> {
        Ok(())
    }

    fn release(&self, _type_: WakeLockType) {}
}
