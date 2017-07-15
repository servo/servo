/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use dom::bindings::str::{ByteString, DOMString};
pub use dom::headers::normalize_value;

// For compile-fail tests only.
pub use dom::bindings::cell::DOMRefCell;
pub use dom::bindings::js::JS;
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
