/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! States elements can be in.

#![deny(missing_docs)]

bitflags! {
    #[doc = "Event-based element states."]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub flags ElementState: u32 {
        #[doc = "The mouse is down on this element. \
                 https://html.spec.whatwg.org/multipage/#selector-active \
                 FIXME(#7333): set/unset this when appropriate"]
        const IN_ACTIVE_STATE = 1 << 0,
        #[doc = "This element has focus. \
                 https://html.spec.whatwg.org/multipage/#selector-focus"]
        const IN_FOCUS_STATE = 1 << 1,
        #[doc = "The mouse is hovering over this element. \
                 https://html.spec.whatwg.org/multipage/#selector-hover"]
        const IN_HOVER_STATE = 1 << 2,
        #[doc = "Content is enabled (and can be disabled). \
                 http://www.whatwg.org/html/#selector-enabled"]
        const IN_ENABLED_STATE = 1 << 3,
        #[doc = "Content is disabled. \
                 http://www.whatwg.org/html/#selector-disabled"]
        const IN_DISABLED_STATE = 1 << 4,
        #[doc = "Content is checked. \
                 https://html.spec.whatwg.org/multipage/#selector-checked"]
        const IN_CHECKED_STATE = 1 << 5,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-indeterminate"]
        const IN_INDETERMINATE_STATE = 1 << 6,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-placeholder-shown"]
        const IN_PLACEHOLDER_SHOWN_STATE = 1 << 7,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-target"]
        const IN_TARGET_STATE = 1 << 8,
        #[doc = "https://fullscreen.spec.whatwg.org/#%3Afullscreen-pseudo-class"]
        const IN_FULLSCREEN_STATE = 1 << 9,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-valid"]
        const IN_VALID_STATE = 1 << 10,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-invalid"]
        const IN_INVALID_STATE = 1 << 11,
        #[doc = "Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-ui-valid"]
        const IN_MOZ_UI_VALID_STATE = 1 << 12,
        #[doc = "Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-broken"]
        const IN_BROKEN_STATE = 1 << 13,
        #[doc = "Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-user-disabled"]
        const IN_USER_DISABLED_STATE = 1 << 14,
        #[doc = "Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-suppressed"]
        const IN_SUPPRESSED_STATE = 1 << 15,
        #[doc = "Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-loading"]
        const IN_LOADING_STATE = 1 << 16,
        #[doc = "Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-handler-blocked"]
        const IN_HANDLER_BLOCKED_STATE = 1 << 17,
        #[doc = "Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-handler-disabled"]
        const IN_HANDLER_DISABLED_STATE = 1 << 18,
        #[doc = "Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-handler-crashed"]
        const IN_HANDLER_CRASHED_STATE = 1 << 19,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-required"]
        const IN_REQUIRED_STATE = 1 << 20,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-optional"]
        const IN_OPTIONAL_STATE = 1 << 21,
        #[doc = "https://html.spec.whatwg.org/multipage/#selector-read-write"]
        const IN_READ_WRITE_STATE = 1 << 22,
    }
}
