/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// For compile-fail tests only.
pub(crate) use crate::dom::bindings::cell::DomRefCell;
pub(crate) use crate::dom::bindings::refcounted::TrustedPromise;
pub(crate) use crate::dom::bindings::root::Dom;
pub(crate) use crate::dom::bindings::str::{ByteString, DOMString};
pub(crate) use crate::dom::headers::normalize_value;
pub(crate) use crate::dom::node::Node;

pub(crate) mod area {
    pub(crate) use crate::dom::htmlareaelement::{Area, Shape};
}

#[allow(non_snake_case)]
pub(crate) mod size_of {
    use std::mem::size_of;

    use crate::dom::characterdata::CharacterData;
    use crate::dom::element::Element;
    use crate::dom::eventtarget::EventTarget;
    use crate::dom::htmldivelement::HTMLDivElement;
    use crate::dom::htmlelement::HTMLElement;
    use crate::dom::htmlspanelement::HTMLSpanElement;
    use crate::dom::node::Node;
    use crate::dom::text::Text;

    pub(crate) fn CharacterData() -> usize {
        size_of::<CharacterData>()
    }

    pub(crate) fn Element() -> usize {
        size_of::<Element>()
    }

    pub(crate) fn EventTarget() -> usize {
        size_of::<EventTarget>()
    }

    pub(crate) fn HTMLDivElement() -> usize {
        size_of::<HTMLDivElement>()
    }

    pub(crate) fn HTMLElement() -> usize {
        size_of::<HTMLElement>()
    }

    pub(crate) fn HTMLSpanElement() -> usize {
        size_of::<HTMLSpanElement>()
    }

    pub(crate) fn Node() -> usize {
        size_of::<Node>()
    }

    pub(crate) fn Text() -> usize {
        size_of::<Text>()
    }
}

pub(crate) mod srcset {
    pub(crate) use crate::dom::htmlimageelement::{parse_a_srcset_attribute, Descriptor, ImageSource};
}

pub(crate) mod timeranges {
    pub(crate) use crate::dom::timeranges::TimeRangesContainer;
}
