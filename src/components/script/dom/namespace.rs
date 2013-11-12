/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_str_as_empty_ref};

#[deriving(Eq, Clone)]
pub enum Namespace {
    Null,
    HTML,
    XML,
    XMLNS,
    XLink,
    SVG,
    MathML,
    Other(~str)
}

impl Namespace {
    pub fn from_str(url: &Option<DOMString>) -> Namespace {
        match null_str_as_empty_ref(url) {
            &"http://www.w3.org/1999/xhtml" => HTML,
            &"http://www.w3.org/XML/1998/namespace" => XML,
            &"http://www.w3.org/2000/xmlns/" => XMLNS,
            &"http://www.w3.org/1999/xlink" => XLink,
            &"http://www.w3.org/2000/svg" => SVG,
            &"http://www.w3.org/1998/Math/MathML" => MathML,
            &"" => Null,
            ns => Other(ns.to_owned())
        }
    }
    pub fn to_str(&self) -> Option<DOMString> {
        match *self {
            Null => None,
            HTML => Some(~"http://www.w3.org/1999/xhtml"),
            XML => Some(~"http://www.w3.org/XML/1998/namespace"),
            XMLNS => Some(~"http://www.w3.org/2000/xmlns/"),
            XLink => Some(~"http://www.w3.org/1999/xlink"),
            SVG => Some(~"http://www.w3.org/2000/svg"),
            MathML => Some(~"http://www.w3.org/1998/Math/MathML"),
            Other(ref x) => Some(x.to_owned())
        }
    }
}
