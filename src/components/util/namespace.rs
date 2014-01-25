/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
    /// Empty string for "no namespace"
    pub fn from_str(url: &str) -> Namespace {
        match url {
            "http://www.w3.org/1999/xhtml" => HTML,
            "http://www.w3.org/XML/1998/namespace" => XML,
            "http://www.w3.org/2000/xmlns/" => XMLNS,
            "http://www.w3.org/1999/xlink" => XLink,
            "http://www.w3.org/2000/svg" => SVG,
            "http://www.w3.org/1998/Math/MathML" => MathML,
            "" => Null,
            ns => Other(ns.to_owned())
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
