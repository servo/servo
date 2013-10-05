/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use extra::url::Url;
use std::cell::Cell;
use newcss::stylesheet::Stylesheet;
use newcss::select::SelectCtx;
use newcss::types::OriginUA;
use newcss::util::DataStream;

pub fn new_css_select_ctx() -> SelectCtx {
    let mut ctx = SelectCtx::new();
    ctx.append_sheet(html4_default_style(), OriginUA);
    ctx.append_sheet(servo_default_style(), OriginUA);
    return ctx;
}

fn html4_default_style() -> Stylesheet {
    Stylesheet::new(default_url("html4_style"),
                    style_stream(html4_default_style_str()))
}

fn servo_default_style() -> Stylesheet {
    Stylesheet::new(default_url("servo_style"),
                    style_stream(servo_default_style_str()))
}

fn default_url(name: &str) -> Url {
    FromStr::from_str(fmt!("http://%s", name)).unwrap()
}

fn style_stream(style: &str) -> DataStream {
    let style = Cell::new(style.as_bytes().to_owned());
    let d: DataStream = || if !style.is_empty() {
        Some(style.take())
    } else {
        None
    };
    return d;
}

fn html4_default_style_str() -> &'static str {
    include_str!("user-agent.css")
}


// FIXME: this shouldn’t be needed.
// The initial value of border-*-width is 'medium' (for which 2px is ok.)
// It’s the *computed values* that is set to 0 when the corresponding
// border-*-style is 'none' (the initial value) or 'hidden'.
// This should be taken care of when removing libcss.
fn servo_default_style_str() -> &'static str {
    // libcss want's this to default to 2px..
    "* { border-width: 0px; }"
}
