/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use string_cache::Atom;

/// Platform specific font representation for Linux.
/// The identifier is an absolute path, and the bytes
/// field is the loaded data that can be passed to
/// freetype and azure directly.
#[derive(Deserialize, Serialize)]
pub struct FontTemplateData {
    pub bytes: Vec<u8>,
    pub identifier: Atom,
}
