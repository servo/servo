/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::SourceLocation;
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_template::FontTemplateDescriptor;
use ipc_channel::ipc;
use style::computed_values::{font_stretch, font_weight};
use style::computed_values::font_family::{FamilyName, FontFamily};
use style::font_face::{FontFaceRuleData, Source};

#[test]
fn test_local_web_font() {
    let (inp_chan, _) = ipc::channel().unwrap();
    let (out_chan, out_receiver) = ipc::channel().unwrap();
    let font_cache_thread = FontCacheThread::new(inp_chan, None);
    let family_name = FamilyName {
        name: From::from("test family"),
        quoted: true,
    };
    let variant_name = FamilyName {
        name: From::from("test font face"),
        quoted: true,
    };
    let font_face_rule = FontFaceRuleData {
        family: Some(family_name.clone()),
        sources: Some(vec![Source::Local(variant_name)]),
        source_location: SourceLocation {
            line: 0,
            column: 0,
        },
    };

    font_cache_thread.add_web_font(
        family_name.clone(),
        font_face_rule.font_face().unwrap().effective_sources(),
        out_chan);

    let font_family = FontFamily::FamilyName(family_name);
    let font_template_descriptor = FontTemplateDescriptor::new(font_weight::T::Weight400,
                                                               font_stretch::T::normal,
                                                               false);
    let result = font_cache_thread.find_font_template(font_family.clone(), font_template_descriptor.clone());
    if let Some(_) = result {
        panic!("Should not have a value since we don't even load it yet.");
    }

    assert_eq!(out_receiver.recv().unwrap(), ());
}
