/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;
use std::iter::Iterator;

use dom_struct::dom_struct;
use servo_atoms::Atom;
use style::custom_properties;

use super::bindings::trace::HashMapTracedValues;
use crate::dom::bindings::codegen::Bindings::StylePropertyMapReadOnlyBinding::StylePropertyMapReadOnlyMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssstylevalue::CSSStyleValue;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct StylePropertyMapReadOnly {
    reflector: Reflector,
    entries: HashMapTracedValues<Atom, Dom<CSSStyleValue>>,
}

impl StylePropertyMapReadOnly {
    fn new_inherited<Entries>(entries: Entries) -> StylePropertyMapReadOnly
    where
        Entries: IntoIterator<Item = (Atom, Dom<CSSStyleValue>)>,
    {
        StylePropertyMapReadOnly {
            reflector: Reflector::new(),
            entries: HashMapTracedValues(entries.into_iter().collect()),
        }
    }

    pub(crate) fn from_iter<Entries>(
        global: &GlobalScope,
        entries: Entries,
        can_gc: CanGc,
    ) -> DomRoot<StylePropertyMapReadOnly>
    where
        Entries: IntoIterator<Item = (Atom, String)>,
    {
        let mut keys = Vec::new();
        rooted_vec!(let mut values);
        let iter = entries.into_iter();
        let (lo, _) = iter.size_hint();
        keys.reserve(lo);
        values.reserve(lo);
        for (key, value) in iter {
            let value = CSSStyleValue::new(global, value, can_gc);
            keys.push(key);
            values.push(Dom::from_ref(&*value));
        }
        let iter = keys.drain(..).zip(values.iter().cloned());
        reflect_dom_object(
            Box::new(StylePropertyMapReadOnly::new_inherited(iter)),
            global,
            can_gc,
        )
    }
}

impl StylePropertyMapReadOnlyMethods<crate::DomTypeHolder> for StylePropertyMapReadOnly {
    /// <https://drafts.css-houdini.org/css-typed-om-1/#dom-stylepropertymapreadonly-get>
    fn Get(&self, property: DOMString) -> Option<DomRoot<CSSStyleValue>> {
        // TODO: avoid constructing an Atom
        self.entries
            .get(&Atom::from(property))
            .map(|value| DomRoot::from_ref(&**value))
    }

    /// <https://drafts.css-houdini.org/css-typed-om-1/#dom-stylepropertymapreadonly-has>
    fn Has(&self, property: DOMString) -> bool {
        // TODO: avoid constructing an Atom
        self.entries.contains_key(&Atom::from(property))
    }

    /// <https://drafts.css-houdini.org/css-typed-om-1/#dom-stylepropertymapreadonly-getproperties>
    fn GetProperties(&self) -> Vec<DOMString> {
        let mut result: Vec<DOMString> = self
            .entries
            .0
            .keys()
            .map(|key| DOMString::from(&**key))
            .collect();
        // https://drafts.css-houdini.org/css-typed-om-1/#dom-stylepropertymap-getproperties
        // requires this sort order
        result.sort_by(|key1, key2| {
            if let Ok(key1) = custom_properties::parse_name(key1) {
                if let Ok(key2) = custom_properties::parse_name(key2) {
                    key1.cmp(key2)
                } else {
                    Ordering::Greater
                }
            } else if custom_properties::parse_name(key2).is_ok() {
                Ordering::Less
            } else {
                key1.cmp(key2)
            }
        });
        result
    }
}
