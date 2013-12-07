/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/* This file exists just to make it easier to import things inside of
 ./text/ without specifying the file they came out of imports.
 
Note that you still must define each of the files as a module in
servo.rc. This is not ideal and may be changed in the future. */

pub use text::shaping::Shaper;
pub use text::text_run::TextRun;

pub mod glyph;
#[path="shaping/mod.rs"] pub mod shaping;
pub mod text_run;
pub mod util;

