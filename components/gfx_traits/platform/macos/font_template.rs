/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core_text::font::CTFont;
use string_cache::Atom;

/// Platform specific font representation for mac.
/// The identifier is a PostScript font name. The
/// CTFont object is cached here for use by the
/// paint functions that create CGFont references.
#[derive(Deserialize, Serialize)]
pub struct FontTemplateData {
    /// The `CTFont` object, if present. This is cached here so that we don't have to keep creating
    /// `CTFont` instances over and over. It can always be recreated from the `identifier` and/or
    /// `font_data` fields.
    ///
    /// When sending a `FontTemplateData` instance across processes, this will be set to `None` on
    /// the other side, because `CTFont` instances cannot be sent across processes. This is
    /// harmless, however, because it can always be recreated.
    pub ctfont: CachedCTFont,

    pub identifier: Atom,
    pub font_data: Option<Vec<u8>>
}

pub struct CachedCTFont(Option<CTFont>);
