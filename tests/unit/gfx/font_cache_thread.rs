/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::SourceLocation;
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::ipc;
use style::computed_values::font_family::FamilyName;
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
        family_name,
        font_face_rule.font_face().unwrap().effective_sources(),
        out_chan);

    assert_eq!(out_receiver.recv().unwrap(), ());
}
