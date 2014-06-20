/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A list of static atoms that are pre-hashed at compile time.

use phf::PhfOrderedMap;
use std::from_str::FromStr;

#[repr(u32)]
#[deriving(Eq, TotalEq)]
pub enum StaticAtom {
	EmptyStringAtom,
	IdAtom,
    ClassAtom,
    HrefAtom,
    StyleAtom,
    SpanAtom,
    WidthAtom,
    HeightAtom,
    TypeAtom,
    DataAtom,
    NewAtom,
    NameAtom,
    SrcAtom,
    RelAtom,
    DivAtom,
}

static STATIC_ATOMS: PhfOrderedMap<StaticAtom> = phf_ordered_map!(
    "" => EmptyStringAtom,
    "id" => IdAtom,
    "class" => ClassAtom,
    "href" => HrefAtom,
    "style" => StyleAtom,
    "span" => SpanAtom,
    "width" => WidthAtom,
    "height" => HeightAtom,
    "type" => TypeAtom,
    "data" => DataAtom,
    "new" => NewAtom,
    "name" => NameAtom,
    "src" => SrcAtom,
    "rel" => RelAtom,
    "div" => DivAtom,
);

impl FromStr for StaticAtom {
    #[inline]
    fn from_str(string: &str) -> Option<StaticAtom> {
        match STATIC_ATOMS.find(&string) {
            None => None,
            Some(&k) => Some(k)
        }
    }
}

impl StaticAtom {
    pub fn as_slice(&self) -> &'static str {
        let (string, _) = STATIC_ATOMS.entries().idx(*self as uint).unwrap();
        string
    }
}
