/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use dom::bindings::str::{ByteString, DOMString};
pub use dom::headers::normalize_value;

// For compile-fail tests only.
pub use dom::bindings::cell::DomRefCell;
pub use dom::bindings::root::Dom;
pub use dom::node::Node;
pub use dom::bindings::refcounted::TrustedPromise;

pub mod area {
    pub use dom::htmlareaelement::{Area, Shape};
}

pub mod size_of {
    use dom::characterdata::CharacterData;
    use dom::element::Element;
    use dom::eventtarget::EventTarget;
    use dom::htmldivelement::HTMLDivElement;
    use dom::htmlelement::HTMLElement;
    use dom::htmlspanelement::HTMLSpanElement;
    use dom::node::Node;
    use dom::text::Text;
    use std::mem::size_of;
    use typeholder::TypeHolderTrait;

    pub fn CharacterData<TH: TypeHolderTrait>() -> usize {
        size_of::<CharacterData<TH>>()
    }

    pub fn Element<TH: TypeHolderTrait>() -> usize {
        size_of::<Element<TH>>()
    }

    pub fn EventTarget<TH: TypeHolderTrait>() -> usize {
        size_of::<EventTarget<TH>>()
    }

    pub fn HTMLDivElement<TH: TypeHolderTrait>() -> usize {
        size_of::<HTMLDivElement<TH>>()
    }

    pub fn HTMLElement<TH: TypeHolderTrait>() -> usize {
        size_of::<HTMLElement<TH>>()
    }

    pub fn HTMLSpanElement<TH: TypeHolderTrait>() -> usize {
        size_of::<HTMLSpanElement<TH>>()
    }

    pub fn Node<TH: TypeHolderTrait>() -> usize {
        size_of::<Node<TH>>()
    }

    pub fn Text<TH: TypeHolderTrait>() -> usize {
        size_of::<Text<TH>>()
    }
}

pub mod srcset {
    pub use dom::htmlimageelement::{parse_a_srcset_attribute, ImageSource, Descriptor};
}
