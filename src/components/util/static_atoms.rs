/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A list of static atoms that are pre-hashed at compile time.

use phf::PhfOrderedMap;

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

#[inline]
pub fn string_to_static_atom(string: &str) -> Option<StaticAtom> {
    match STATIC_ATOMS.find(&string) {
    	None => None,
    	Some(&k) => Some(k)
    }
}

#[inline]
pub fn static_atom_to_string(atom_id: StaticAtom) -> &'static str {
	let (string, _) = STATIC_ATOMS.entries().idx(atom_id as uint).unwrap();
	string
}
