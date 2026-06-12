/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::any::Any;

/// Opaque handle to a backend-compiled model.
///
/// Each backend owns its internal representation; the rest of the
/// system treats this as a token to pass back to `Backend::run`.
pub struct CompiledModel(pub Box<dyn Any + Send + Sync>);

/// Output buffers produced by one call to `Backend::run`.
pub struct RunResult {
    pub outputs: Vec<Vec<u8>>,
}
