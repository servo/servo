/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use extra::url::Url;
use std::cell::Cell;
use style::Stylesheet;
use style::Stylist;
use style::selector_matching::UserAgentOrigin;
use newcss::util::DataStream;

pub fn new_stylist() -> Stylist {
    let mut stylist = Stylist::new();
    stylist.add_stylesheet(html4_default_style(), UserAgentOrigin);
    stylist.add_stylesheet(servo_default_style(), UserAgentOrigin);
    stylist
}

fn html4_default_style() -> Stylesheet {
    Stylesheet::from_str(html4_default_style_str())
}

fn servo_default_style() -> Stylesheet {
    Stylesheet::from_str(servo_default_style_str())
}

fn default_url(name: &str) -> Url {
    FromStr::from_str(fmt!("http://%s", name)).unwrap()
}

fn style_stream(style: &str) -> @mut DataStream {
    let style = Cell::new(style.as_bytes().to_owned());
    struct StyleDataStream {
        style: Cell<~[u8]>,
    }
    impl DataStream for StyleDataStream {
        fn read(&mut self) -> Option<~[u8]> {
            if !self.style.is_empty() {
                Some(self.style.take())
            } else {
                None
            }
        }
    }
    let stream = @mut StyleDataStream {
        style: style,
    };
    stream as @mut DataStream
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
