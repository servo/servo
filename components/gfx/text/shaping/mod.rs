/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Shaper encapsulates a specific shaper, such as Harfbuzz,
//! Uniscribe, Pango, or Coretext.
//!
//! Currently, only harfbuzz bindings are implemented.

use text::glyph::GlyphStore;

pub use text::shaping::harfbuzz::Shaper;

pub mod harfbuzz;

pub trait ShaperMethods {
    fn shape_text(&self, text: &str, glyphs: &mut GlyphStore);
}

