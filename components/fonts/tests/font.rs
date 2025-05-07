/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use app_units::Au;
use euclid::num::Zero;
use fonts::platform::font::PlatformFont;
use fonts::{
    Font, FontData, FontDescriptor, FontIdentifier, FontTemplate, FontTemplateRef,
    PlatformFontMethods, ShapingFlags, ShapingOptions,
};
use servo_url::ServoUrl;
use style::properties::longhands::font_variant_caps::computed_value::T as FontVariantCaps;
use style::values::computed::{FontStretch, FontStyle, FontWeight};
use unicode_script::Script;

fn make_font(path: PathBuf) -> Font {
    let mut bytes = Vec::new();
    File::open(path.clone())
        .expect("Couldn't open font file!")
        .read_to_end(&mut bytes)
        .unwrap();
    let data = FontData::from_bytes(&bytes);

    let identifier = FontIdentifier::Web(ServoUrl::from_file_path(path).unwrap());
    let platform_font = PlatformFont::new_from_data(identifier.clone(), &data, None).unwrap();

    let template = FontTemplate {
        identifier,
        descriptor: platform_font.descriptor(),
        stylesheet: None,
    };
    let descriptor = FontDescriptor {
        weight: FontWeight::normal(),
        stretch: FontStretch::hundred(),
        style: FontStyle::normal(),
        variant: FontVariantCaps::Normal,
        pt_size: Au::from_px(24),
    };
    Font::new(FontTemplateRef::new(template), descriptor, Some(data), None).unwrap()
}

#[test]
fn test_font_can_do_fast_shaping() {
    let dejavu_sans = make_font(
        [
            env!("CARGO_MANIFEST_DIR"),
            "tests",
            "support",
            "dejavu-fonts-ttf-2.37",
            "ttf",
            "DejaVuSans.ttf",
        ]
        .iter()
        .collect(),
    );

    let dejavu_sans_fast_shapeable = make_font(
        [
            env!("CARGO_MANIFEST_DIR"),
            "tests",
            "support",
            "dejavu-fonts-ttf-2.37",
            "ttf",
            "DejaVuSansNoGSUBNoGPOS.ttf",
        ]
        .iter()
        .collect(),
    );

    // Fast shaping requires a font with a kern table and no GPOS or GSUB tables.
    let shaping_options = ShapingOptions {
        letter_spacing: None,
        word_spacing: Au::zero(),
        script: Script::Latin,
        flags: ShapingFlags::empty(),
    };
    assert!(!dejavu_sans.can_do_fast_shaping("WAVE", &shaping_options));
    assert!(dejavu_sans_fast_shapeable.can_do_fast_shaping("WAVE", &shaping_options));

    // Non-Latin script should never have fast shaping.
    let shaping_options = ShapingOptions {
        letter_spacing: None,
        word_spacing: Au::zero(),
        script: Script::Cherokee,
        flags: ShapingFlags::empty(),
    };
    assert!(!dejavu_sans.can_do_fast_shaping("WAVE", &shaping_options));
    assert!(!dejavu_sans_fast_shapeable.can_do_fast_shaping("WAVE", &shaping_options));

    // Right-to-left text should never use fast shaping.
    let shaping_options = ShapingOptions {
        letter_spacing: None,
        word_spacing: Au::zero(),
        script: Script::Latin,
        flags: ShapingFlags::RTL_FLAG,
    };
    assert!(!dejavu_sans.can_do_fast_shaping("WAVE", &shaping_options));
    assert!(!dejavu_sans_fast_shapeable.can_do_fast_shaping("WAVE", &shaping_options));
}
