/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::{Stylesheet, Stylist, UserAgentOrigin, with_errors_silenced};


pub fn new_stylist() -> Stylist {
    let mut stylist = Stylist::new();
    let ua_stylesheet = with_errors_silenced(
        || Stylesheet::from_str(include_str!("user-agent.css")));
    stylist.add_stylesheet(ua_stylesheet, UserAgentOrigin);
    stylist
}
