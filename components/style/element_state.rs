/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! States elements can be in.

bitflags! {
    /// Event-based element states.
    ///
    /// NB: Is important for this to remain in sync with Gecko's
    /// dom/events/EventStates.h.
    ///
    /// Please keep in that order in order for this to be easily auditable.
    ///
    /// TODO(emilio): We really really want to use the NS_EVENT_STATE bindings
    /// for this.
    #[derive(MallocSizeOf)]
    pub struct ElementState: u64 {
        /// The mouse is down on this element.
        /// <https://html.spec.whatwg.org/multipage/#selector-active>
        /// FIXME(#7333): set/unset this when appropriate
        const IN_ACTIVE_STATE = 1 << 0;
        /// This element has focus.
        /// <https://html.spec.whatwg.org/multipage/#selector-focus>
        const IN_FOCUS_STATE = 1 << 1;
        /// The mouse is hovering over this element.
        /// <https://html.spec.whatwg.org/multipage/#selector-hover>
        const IN_HOVER_STATE = 1 << 2;
        /// Content is enabled (and can be disabled).
        /// <http://www.whatwg.org/html/#selector-enabled>
        const IN_ENABLED_STATE = 1 << 3;
        /// Content is disabled.
        /// <http://www.whatwg.org/html/#selector-disabled>
        const IN_DISABLED_STATE = 1 << 4;
        /// Content is checked.
        /// <https://html.spec.whatwg.org/multipage/#selector-checked>
        const IN_CHECKED_STATE = 1 << 5;
        /// <https://html.spec.whatwg.org/multipage/#selector-indeterminate>
        const IN_INDETERMINATE_STATE = 1 << 6;
        /// <https://html.spec.whatwg.org/multipage/#selector-placeholder-shown>
        const IN_PLACEHOLDER_SHOWN_STATE = 1 << 7;
        /// <https://html.spec.whatwg.org/multipage/#selector-target>
        const IN_TARGET_STATE = 1 << 8;
        /// <https://fullscreen.spec.whatwg.org/#%3Afullscreen-pseudo-class>
        const IN_FULLSCREEN_STATE = 1 << 9;
        /// <https://html.spec.whatwg.org/multipage/#selector-valid>
        const IN_VALID_STATE = 1 << 10;
        /// <https://html.spec.whatwg.org/multipage/#selector-invalid>
        const IN_INVALID_STATE = 1 << 11;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-ui-valid
        const IN_MOZ_UI_VALID_STATE = 1 << 12;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-ui-invalid
        const IN_MOZ_UI_INVALID_STATE = 1 << 13;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-broken
        const IN_BROKEN_STATE = 1 << 14;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-user-disabled
        const IN_USER_DISABLED_STATE = 1 << 15;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-suppressed
        const IN_SUPPRESSED_STATE = 1 << 16;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-loading
        const IN_LOADING_STATE = 1 << 17;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-handler-blocked
        const IN_HANDLER_BLOCKED_STATE = 1 << 18;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-handler-disabled
        const IN_HANDLER_DISABLED_STATE = 1 << 19;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-handler-crashed
        const IN_HANDLER_CRASHED_STATE = 1 << 20;
        /// <https://html.spec.whatwg.org/multipage/#selector-required>
        const IN_REQUIRED_STATE = 1 << 21;
        /// <https://html.spec.whatwg.org/multipage/#selector-optional>
        const IN_OPTIONAL_STATE = 1 << 22;
        /// <https://html.spec.whatwg.org/multipage/#selector-read-write>
        const IN_READ_WRITE_STATE = 1 << 22;
        /// <https://html.spec.whatwg.org/multipage/semantics-other.html#selector-defined>
        const IN_DEFINED_STATE = 1 << 23;
        /// <https://html.spec.whatwg.org/multipage/#selector-visited>
        const IN_VISITED_STATE = 1 << 24;
        /// <https://html.spec.whatwg.org/multipage/#selector-link>
        const IN_UNVISITED_STATE = 1 << 25;
        /// <https://drafts.csswg.org/selectors-4/#the-any-link-pseudo>
        const IN_VISITED_OR_UNVISITED_STATE = ElementState::IN_VISITED_STATE.bits |
                                              ElementState::IN_UNVISITED_STATE.bits;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-drag-over
        const IN_DRAGOVER_STATE = 1 << 26;
        /// <https://html.spec.whatwg.org/multipage/#selector-in-range>
        const IN_INRANGE_STATE = 1 << 27;
        /// <https://html.spec.whatwg.org/multipage/#selector-out-of-range>
        const IN_OUTOFRANGE_STATE = 1 << 28;
        /// <https://html.spec.whatwg.org/multipage/#selector-read-only>
        const IN_MOZ_READONLY_STATE = 1 << 29;
        /// <https://html.spec.whatwg.org/multipage/#selector-read-write>
        const IN_MOZ_READWRITE_STATE = 1 << 30;
        /// <https://html.spec.whatwg.org/multipage/#selector-default>
        const IN_DEFAULT_STATE = 1 << 31;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-submit-invalid
        const IN_MOZ_SUBMITINVALID_STATE = 1 << 32;
        /// Non-standard & undocumented.
        const IN_OPTIMUM_STATE = 1 << 33;
        /// Non-standard & undocumented.
        const IN_SUB_OPTIMUM_STATE = 1 << 34;
        /// Non-standard & undocumented.
        const IN_SUB_SUB_OPTIMUM_STATE = 1 << 35;
        /// Non-standard & undocumented.
        const IN_DEVTOOLS_HIGHLIGHTED_STATE = 1 << 36;
        /// Non-standard & undocumented.
        const IN_STYLEEDITOR_TRANSITIONING_STATE = 1 << 37;
        /// Non-standard & undocumented.
        const IN_INCREMENT_SCRIPT_LEVEL_STATE = 1 << 38;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-focusring
        const IN_FOCUSRING_STATE = 1 << 39;
        /// Non-standard & undocumented.
        const IN_HANDLER_CLICK_TO_PLAY_STATE = 1 << 40;
        /// Non-standard & undocumented.
        const IN_HANDLER_VULNERABLE_UPDATABLE_STATE = 1 << 41;
        /// Non-standard & undocumented.
        const IN_HANDLER_VULNERABLE_NO_UPDATE_STATE = 1 << 42;
        /// <https://drafts.csswg.org/selectors-4/#the-focus-within-pseudo>
        const IN_FOCUS_WITHIN_STATE = 1 << 43;
        /// :dir matching; the states are used for dynamic change detection.
        /// State that elements that match :dir(ltr) are in.
        const IN_LTR_STATE = 1 << 44;
        /// State that elements that match :dir(rtl) are in.
        const IN_RTL_STATE = 1 << 45;
        /// State that HTML elements that have a "dir" attr are in.
        const IN_HAS_DIR_ATTR_STATE = 1 << 46;
        /// State that HTML elements with dir="ltr" (or something
        /// case-insensitively equal to "ltr") are in.
        const IN_HAS_DIR_ATTR_LTR_STATE = 1 << 47;
        /// State that HTML elements with dir="rtl" (or something
        /// case-insensitively equal to "rtl") are in.
        const IN_HAS_DIR_ATTR_RTL_STATE = 1 << 48;
        /// State that HTML <bdi> elements without a valid-valued "dir" attr or
        /// any HTML elements (including <bdi>) with dir="auto" (or something
        /// case-insensitively equal to "auto") are in.
        const IN_HAS_DIR_ATTR_LIKE_AUTO_STATE = 1 << 49;
        /// Non-standard & undocumented.
        const IN_AUTOFILL_STATE = 1 << 50;
        /// Non-standard & undocumented.
        const IN_AUTOFILL_PREVIEW_STATE = 1 << 51;
    }
}

bitflags! {
    /// Event-based document states.
    ///
    /// NB: Is important for this to remain in sync with Gecko's
    /// dom/base/nsIDocument.h.
    #[derive(MallocSizeOf)]
    pub struct DocumentState: u64 {
        /// RTL locale: specific to the XUL localedir attribute
        const NS_DOCUMENT_STATE_RTL_LOCALE = 1 << 0;
        /// Window activation status
        const NS_DOCUMENT_STATE_WINDOW_INACTIVE = 1 << 1;
    }
}
