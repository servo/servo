/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use util::opts;
use util::str::DOMString;

use std::borrow::ToOwned;

pub fn Product() -> DOMString {
    "Gecko".to_owned()
}

pub fn TaintEnabled() -> bool {
    false
}

pub fn AppName() -> DOMString {
    "Netscape".to_owned() // Like Gecko/Webkit
}

pub fn AppCodeName() -> DOMString {
    "Mozilla".to_owned()
}

pub fn Platform() -> DOMString {
    "".to_owned()
}

pub fn UserAgent() -> DOMString {
    opts::get().user_agent.clone()
}

pub fn AppVersion() -> DOMString {
    "4.0".to_owned()
}
