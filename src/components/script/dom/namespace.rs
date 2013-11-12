/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::DOMString;

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
    pub fn from_str(url: DOMString) -> Namespace {
        if url.is_some() {
            match url {
                Some(~"http://www.w3.org/1999/xhtml") => HTML,
                Some(~"http://www.w3.org/XML/1998/namespace") => XML,
                Some(~"http://www.w3.org/2000/xmlns/") => XMLNS,
                Some(~"http://www.w3.org/1999/xlink") => XLink,
                Some(~"http://www.w3.org/2000/svg") => SVG,
                Some(~"http://www.w3.org/1998/Math/MathML") => MathML,
                Some(~"") => Null,
                _ => Other(url.unwrap().to_owned())
            }
        }else {
            return Null;
        }
    }
    pub fn to_str(&self) -> DOMString {
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
