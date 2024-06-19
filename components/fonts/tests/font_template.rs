/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Test doesn't yet run on Mac, see https://github.com/servo/servo/pull/19928 for explanation.
#[cfg(not(target_os = "macos"))]
#[test]
fn test_font_template_descriptor() {
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    use fonts::platform::font::PlatformFont;
    use fonts::{FontIdentifier, FontTemplateDescriptor, PlatformFontMethods};
    use servo_url::ServoUrl;
    use style::values::computed::font::{FontStretch, FontStyle, FontWeight};

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

        let identifier = FontIdentifier::Web(ServoUrl::from_file_path(path.clone()).unwrap());
        let file = File::open(path.clone()).unwrap();
        let data = file.bytes().map(|b| b.unwrap()).collect();
        let handle = PlatformFont::new_from_data(identifier, Arc::new(data), 0, None).unwrap();
        handle.descriptor()
    }

    assert_eq!(
        descriptor("DejaVuSans"),
        FontTemplateDescriptor::new(
            FontWeight::NORMAL,
            FontStretch::hundred(),
            FontStyle::NORMAL,
        )
    );

    assert_eq!(
        descriptor("DejaVuSans-Bold"),
        FontTemplateDescriptor::new(FontWeight::BOLD, FontStretch::hundred(), FontStyle::NORMAL,)
    );

    assert_eq!(
        descriptor("DejaVuSans-Oblique"),
        FontTemplateDescriptor::new(
            FontWeight::NORMAL,
            FontStretch::hundred(),
            FontStyle::ITALIC,
        )
    );

    assert_eq!(
        descriptor("DejaVuSansCondensed-BoldOblique"),
        FontTemplateDescriptor::new(
            FontWeight::BOLD,
            FontStretch::from_percentage(0.875),
            FontStyle::ITALIC,
        )
    );
}
