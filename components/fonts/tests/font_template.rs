/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

use fonts::FontTemplateDescriptor;
use skrifa::Font as SkrifaFont;
use skrifa::font::Blob;
use style::values::computed::font::{FontStyle, FontWeight, FontWidth};

#[test]
fn test_font_template_descriptor() {
    fn descriptor(filename: &str) -> FontTemplateDescriptor {
        let mut path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "tests",
            "support",
            "dejavu-fonts-ttf-2.37",
            "ttf",
        ]
        .iter()
        .collect();
        path.push(format!("{}.ttf", filename));

        let mut bytes = Vec::new();
        File::open(path.clone())
            .expect("Couldn't open font file!")
            .read_to_end(&mut bytes)
            .unwrap();
        let skrifa_font = SkrifaFont::new(Blob::Shared(Arc::new(bytes)), 0).unwrap();
        FontTemplateDescriptor::from_skrifa_font(&skrifa_font)
    }

    assert_eq!(
        descriptor("DejaVuSans"),
        FontTemplateDescriptor::new(FontWeight::NORMAL, FontWidth::hundred(), FontStyle::NORMAL)
    );

    assert_eq!(
        descriptor("DejaVuSans-Bold"),
        FontTemplateDescriptor::new(FontWeight::BOLD, FontWidth::hundred(), FontStyle::NORMAL)
    );

    assert_eq!(
        descriptor("DejaVuSans-Oblique"),
        FontTemplateDescriptor::new(FontWeight::NORMAL, FontWidth::hundred(), FontStyle::ITALIC)
    );

    assert_eq!(
        descriptor("DejaVuSansCondensed-BoldOblique"),
        FontTemplateDescriptor::new(
            FontWeight::BOLD,
            FontWidth::from_percentage(0.875),
            FontStyle::ITALIC,
        )
    );
}
