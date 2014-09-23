/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use str::DOMString;

#[deriving(Eq, PartialEq, Clone, Encodable, Hash, Show)]
pub enum Namespace {
    Null,
    HTML,
    XML,
    XMLNS,
    XLink,
    SVG,
    MathML,
    Other(String)
}

impl Namespace {
    /// Empty string for "no namespace"
    pub fn from_str(url: Option<DOMString>) -> Namespace {
        match url {
            None => Null,
            Some(ref ns) if ns.as_slice() == "" => Null,
            Some(ref ns) if ns.as_slice() == "http://www.w3.org/1999/xhtml" => HTML,
            Some(ref ns) if ns.as_slice() == "http://www.w3.org/XML/1998/namespace" => XML,
            Some(ref ns) if ns.as_slice() == "http://www.w3.org/2000/xmlns/" => XMLNS,
            Some(ref ns) if ns.as_slice() == "http://www.w3.org/1999/xlink" => XLink,
            Some(ref ns) if ns.as_slice() == "http://www.w3.org/2000/svg" => SVG,
            Some(ref ns) if ns.as_slice() == "http://www.w3.org/1998/Math/MathML" => MathML,
            Some(ns) => Other(ns)
        }
    }
    pub fn to_str<'a>(&'a self) -> &'a str {
        match *self {
            Null => "",
            HTML => "http://www.w3.org/1999/xhtml",
            XML => "http://www.w3.org/XML/1998/namespace",
            XMLNS => "http://www.w3.org/2000/xmlns/",
            XLink => "http://www.w3.org/1999/xlink",
            SVG => "http://www.w3.org/2000/svg",
            MathML => "http://www.w3.org/1998/Math/MathML",
            Other(ref x) => x.as_slice()
        }
    }
}
