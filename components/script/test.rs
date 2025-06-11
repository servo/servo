/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// For compile-fail tests only.
//pub use crate::dom::bindings::cell::DomRefCell;
pub use crate::dom::bindings::refcounted::TrustedPromise;
//pub use crate::dom::bindings::root::Dom;
pub use crate::dom::bindings::str::{ByteString, DOMString};
//pub use crate::dom::node::Node;

pub mod area {
    pub use crate::dom::htmlareaelement::{Area, Shape};
}

#[allow(non_snake_case)]
pub mod size_of {
    use std::mem::size_of;

    use crate::dom::characterdata::CharacterData;
    use crate::dom::element::Element;
    use crate::dom::eventtarget::EventTarget;
    use crate::dom::htmldivelement::HTMLDivElement;
    use crate::dom::htmlelement::HTMLElement;
    use crate::dom::htmlspanelement::HTMLSpanElement;
    use crate::dom::node::Node;
    use crate::dom::text::Text;

    pub fn CharacterData() -> usize {
        size_of::<CharacterData>()
    }

    pub fn Element() -> usize {
        size_of::<Element>()
    }

    pub fn EventTarget() -> usize {
        size_of::<EventTarget>()
    }

    pub fn HTMLDivElement() -> usize {
        size_of::<HTMLDivElement>()
    }

    pub fn HTMLElement() -> usize {
        size_of::<HTMLElement>()
    }

    pub fn HTMLSpanElement() -> usize {
        size_of::<HTMLSpanElement>()
    }

    pub fn Node() -> usize {
        size_of::<Node>()
    }

    pub fn Text() -> usize {
        size_of::<Text>()
    }
}

pub mod srcset {
    pub use crate::dom::htmlimageelement::{Descriptor, ImageSource, parse_a_srcset_attribute};
}

pub mod timeranges {
    pub use crate::dom::timeranges::TimeRangesContainer;
}

pub mod textinput {
    pub use crate::clipboard_provider::ClipboardProvider;
    pub use crate::textinput::{
        Direction, Lines, Selection, SelectionDirection, TextInput, TextPoint, UTF16CodeUnits,
    };
}
