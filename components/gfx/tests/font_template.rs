/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Test doesn't yet run on Mac, see https://github.com/servo/servo/pull/19928 for explanation.
#[cfg(not(target_os = "macos"))]
#[test]
fn test_font_template_descriptor() {
    use gfx::font_context::FontContextHandle;
    use gfx::font_template::{FontTemplate, FontTemplateDescriptor};
    use servo_atoms::Atom;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::PathBuf;
    use style::values::computed::font::{FontStretch, FontWeight};
    use style::values::computed::Percentage;
    use style::values::generics::font::FontStyle;
    use style::values::generics::NonNegative;

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

        let file = File::open(path).unwrap();

        let mut template = FontTemplate::new(
            Atom::from(filename),
            Some(file.bytes().map(|b| b.unwrap()).collect()),
        )
        .unwrap();

        let context = FontContextHandle::new();

        template.descriptor(&context).unwrap()
    }

    assert_eq!(
        descriptor("DejaVuSans"),
        FontTemplateDescriptor {
            weight: FontWeight::normal(),
            stretch: FontStretch::hundred(),
            style: FontStyle::Normal,
        }
    );

    assert_eq!(
        descriptor("DejaVuSans-Bold"),
        FontTemplateDescriptor {
            weight: FontWeight::bold(),
            stretch: FontStretch::hundred(),
            style: FontStyle::Normal,
        }
    );

    assert_eq!(
        descriptor("DejaVuSans-Oblique"),
        FontTemplateDescriptor {
            weight: FontWeight::normal(),
            stretch: FontStretch::hundred(),
            style: FontStyle::Italic,
        }
    );

    assert_eq!(
        descriptor("DejaVuSansCondensed-BoldOblique"),
        FontTemplateDescriptor {
            weight: FontWeight::bold(),
            stretch: FontStretch(NonNegative(Percentage(0.875))),
            style: FontStyle::Italic,
        }
    );
}
