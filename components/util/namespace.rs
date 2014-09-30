/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use str::DOMString;
use string_cache::{Atom, Namespace};

pub fn from_domstring(url: Option<DOMString>) -> Namespace {
    match url {
        None => ns!(""),
        Some(ref s) => Namespace(Atom::from_slice(s.as_slice())),
    }
}
