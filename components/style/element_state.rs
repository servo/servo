/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! States elements can be in.

#![deny(missing_docs)]

bitflags! {
    #[doc = "Event-based element states."]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub flags ElementState: u16 {
        #[doc = "The mouse is down on this element. \
                 https://html.spec.whatwg.org/multipage/#selector-active \
                 FIXME(#7333): set/unset this when appropriate"]
        const IN_ACTIVE_STATE = 0x01,
        #[doc = "This element has focus. \
                 https://html.spec.whatwg.org/multipage/#selector-focus"]
        const IN_FOCUS_STATE = 0x02,
        #[doc = "The mouse is hovering over this element. \
                 https://html.spec.whatwg.org/multipage/#selector-hover"]
        const IN_HOVER_STATE = 0x04,
        #[doc = "Content is enabled (and can be disabled). \
                 http://www.whatwg.org/html/#selector-enabled"]
        const IN_ENABLED_STATE = 0x08,
        #[doc = "Content is disabled. \
                 http://www.whatwg.org/html/#selector-disabled"]
        const IN_DISABLED_STATE = 0x10,
        #[doc = "Content is checked. \
                 https://html.spec.whatwg.org/multipage/#selector-checked"]
        const IN_CHECKED_STATE = 0x20,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-indeterminate"]
        const IN_INDETERMINATE_STATE = 0x40,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-read-write"]
        const IN_READ_WRITE_STATE = 0x80,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-placeholder-shown"]
        const IN_PLACEHOLDER_SHOWN_STATE = 0x0100,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-target"]
        const IN_TARGET_STATE = 0x0200,
        #[doc = "https://fullscreen.spec.whatwg.org/#%3Afullscreen-pseudo-class"]
        const IN_FULLSCREEN_STATE = 0x0400,
    }
}
