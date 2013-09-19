/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{null_string, str, DOMString};

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
    pub fn from_str(url: &str) -> Namespace {
        match url {
            "http://www.w3.org/1999/xhtml" => HTML,
            "http://www.w3.org/XML/1998/namespace" => XML,
            "http://www.w3.org/2000/xmlns/" => XMLNS,
            "http://www.w3.org/1999/xlink" => XLink,
            "http://www.w3.org/2000/svg" => SVG,
            "http://www.w3.org/1998/Math/MathML" => MathML,
            _ => Other(url.to_owned())
        }
    }
    pub fn to_str(&self) -> DOMString {
        match *self {
            Null => null_string,
            HTML => str(~"http://www.w3.org/1999/xhtml"),
            XML => str(~"http://www.w3.org/XML/1998/namespace"),
            XMLNS => str(~"http://www.w3.org/2000/xmlns/"),
            XLink => str(~"http://www.w3.org/1999/xlink"),
            SVG => str(~"http://www.w3.org/2000/svg"),
            MathML => str(~"http://www.w3.org/1998/Math/MathML"),
            Other(ref x) => str(x.to_owned())
        }
    }
}
