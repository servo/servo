/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate gfx;
extern crate servo_atoms;
extern crate style;

use gfx::font_context::FontContextHandle;
use gfx::font_template::{FontTemplate, FontTemplateDescriptor};
use servo_atoms::Atom;
use style::values::computed::font::FontWeight;
use style::computed_values::font_stretch::T as FontStretch;
use std::path::PathBuf;

fn descriptor(filename: &str) -> FontTemplateDescriptor {
    let mut path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "support",
        "dejavu-fonts-ttf-2.37",
        "ttf",
    ].iter().collect();
    path.push(format!("{}.ttf", filename));

    let path = path.to_str().unwrap();

    let mut template = FontTemplate::new(Atom::from(path), None).unwrap();
    let context = FontContextHandle::new();

    template.descriptor(&context).unwrap()
}

#[test]
fn test_font_template_descriptor() {
    assert_eq!(descriptor("DejaVuSans"), FontTemplateDescriptor {
        weight: FontWeight::normal(),
        stretch: FontStretch::Normal,
        italic: false,
    });

    assert_eq!(descriptor("DejaVuSans-Bold"), FontTemplateDescriptor {
        weight: FontWeight::bold(),
        stretch: FontStretch::Normal,
        italic: false,
    });

    assert_eq!(descriptor("DejaVuSans-Oblique"), FontTemplateDescriptor {
        weight: FontWeight::normal(),
        stretch: FontStretch::Normal,
        italic: true,
    });

    assert_eq!(descriptor("DejaVuSansCondensed-BoldOblique"), FontTemplateDescriptor {
        weight: FontWeight::bold(),
        stretch: FontStretch::SemiCondensed,
        italic: true,
    });
}
