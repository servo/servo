/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Types used to access the DOM from style calculation.

use bitflags::bitflags;
use malloc_size_of::malloc_size_of_is_0;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

/// An opaque handle to a node, which, unlike UnsafeNode, cannot be transformed
/// back into a non-opaque representation. The only safe operation that can be
/// performed on this node is to compare it to another opaque handle or to another
/// OpaqueNode.
///
/// Layout and Graphics use this to safely represent nodes for comparison purposes.
/// Because the script task's GC does not trace layout, node data cannot be safely stored in layout
/// data structures. Also, layout code tends to be faster when the DOM is not being accessed, for
/// locality reasons. Using `OpaqueNode` enforces this invariant.
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct OpaqueNode(pub usize);

impl OpaqueNode {
    /// Returns the address of this node, for debugging purposes.
    #[inline]
    pub fn id(&self) -> usize {
        self.0
    }
}

// DOM types to be shared between Rust and C++.

bitflags! {
    /// Event-based element states.
    #[repr(C)]
    pub struct ElementState: u64 {
        /// The mouse is down on this element.
        /// <https://html.spec.whatwg.org/multipage/#selector-active>
        /// FIXME(#7333): set/unset this when appropriate
        const ACTIVE = 1 << 0;
        /// This element has focus.
        /// <https://html.spec.whatwg.org/multipage/#selector-focus>
        const FOCUS = 1 << 1;
        /// The mouse is hovering over this element.
        /// <https://html.spec.whatwg.org/multipage/#selector-hover>
        const HOVER = 1 << 2;
        /// Content is enabled (and can be disabled).
        /// <http://www.whatwg.org/html/#selector-enabled>
        const ENABLED = 1 << 3;
        /// Content is disabled.
        /// <http://www.whatwg.org/html/#selector-disabled>
        const DISABLED = 1 << 4;
        /// Content is checked.
        /// <https://html.spec.whatwg.org/multipage/#selector-checked>
        const CHECKED = 1 << 5;
        /// <https://html.spec.whatwg.org/multipage/#selector-indeterminate>
        const INDETERMINATE = 1 << 6;
        /// <https://html.spec.whatwg.org/multipage/#selector-placeholder-shown>
        const PLACEHOLDER_SHOWN = 1 << 7;
        /// <https://html.spec.whatwg.org/multipage/#selector-target>
        const URLTARGET = 1 << 8;
        /// <https://fullscreen.spec.whatwg.org/#%3Afullscreen-pseudo-class>
        const FULLSCREEN = 1 << 9;
        /// <https://html.spec.whatwg.org/multipage/#selector-valid>
        const VALID = 1 << 10;
        /// <https://html.spec.whatwg.org/multipage/#selector-invalid>
        const INVALID = 1 << 11;
        /// <https://drafts.csswg.org/selectors-4/#user-valid-pseudo>
        const USER_VALID = 1 << 12;
        /// <https://drafts.csswg.org/selectors-4/#user-invalid-pseudo>
        const USER_INVALID = 1 << 13;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-broken
        const BROKEN = 1 << 14;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-loading
        const LOADING = 1 << 15;
        /// <https://html.spec.whatwg.org/multipage/#selector-required>
        const REQUIRED = 1 << 16;
        /// <https://html.spec.whatwg.org/multipage/#selector-optional>
        /// We use an underscore to workaround a silly windows.h define.
        const OPTIONAL_ = 1 << 17;
        /// <https://html.spec.whatwg.org/multipage/#selector-defined>
        const DEFINED = 1 << 18;
        /// <https://html.spec.whatwg.org/multipage/#selector-visited>
        const VISITED = 1 << 19;
        /// <https://html.spec.whatwg.org/multipage/#selector-link>
        const UNVISITED = 1 << 20;
        /// <https://drafts.csswg.org/selectors-4/#the-any-link-pseudo>
        const VISITED_OR_UNVISITED = Self::VISITED.bits | Self::UNVISITED.bits;
        /// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/:-moz-drag-over
        const DRAGOVER = 1 << 21;
        /// <https://html.spec.whatwg.org/multipage/#selector-in-range>
        const INRANGE = 1 << 22;
        /// <https://html.spec.whatwg.org/multipage/#selector-out-of-range>
        const OUTOFRANGE = 1 << 23;
        /// <https://html.spec.whatwg.org/multipage/#selector-read-only>
        const READONLY = 1 << 24;
        /// <https://html.spec.whatwg.org/multipage/#selector-read-write>
        const READWRITE = 1 << 25;
        /// <https://html.spec.whatwg.org/multipage/#selector-default>
        const DEFAULT = 1 << 26;
        /// Non-standard & undocumented.
        const OPTIMUM = 1 << 28;
        /// Non-standard & undocumented.
        const SUB_OPTIMUM = 1 << 29;
        /// Non-standard & undocumented.
        const SUB_SUB_OPTIMUM = 1 << 30;
        /// Non-standard & undocumented.
        const INCREMENT_SCRIPT_LEVEL = 1u64 << 31;
        /// <https://drafts.csswg.org/selectors-4/#the-focus-visible-pseudo>
        const FOCUSRING = 1u64 << 32;
        /// <https://drafts.csswg.org/selectors-4/#the-focus-within-pseudo>
        const FOCUS_WITHIN = 1u64 << 33;
        /// :dir matching; the states are used for dynamic change detection.
        /// State that elements that match :dir(ltr) are in.
        const LTR = 1u64 << 34;
        /// State that elements that match :dir(rtl) are in.
        const RTL = 1u64 << 35;
        /// State that HTML elements that have a "dir" attr are in.
        const HAS_DIR_ATTR = 1u64 << 36;
        /// State that HTML elements with dir="ltr" (or something
        /// case-insensitively equal to "ltr") are in.
        const HAS_DIR_ATTR_LTR = 1u64 << 37;
        /// State that HTML elements with dir="rtl" (or something
        /// case-insensitively equal to "rtl") are in.
        const HAS_DIR_ATTR_RTL = 1u64 << 38;
        /// State that HTML <bdi> elements without a valid-valued "dir" attr or
        /// any HTML elements (including <bdi>) with dir="auto" (or something
        /// case-insensitively equal to "auto") are in.
        const HAS_DIR_ATTR_LIKE_AUTO = 1u64 << 39;
        /// Non-standard & undocumented.
        const AUTOFILL = 1u64 << 40;
        /// Non-standard & undocumented.
        const AUTOFILL_PREVIEW = 1u64 << 41;
        /// State that dialog element is modal, for centered alignment
        /// <https://html.spec.whatwg.org/multipage/#centered-alignment>
        const MODAL_DIALOG = 1u64 << 42;
        /// <https://html.spec.whatwg.org/multipage/#inert-subtrees>
        const INERT = 1u64 << 43;
        /// State for the topmost modal element in top layer
        const TOPMOST_MODAL = 1u64 << 44;
        /// Initially used for the devtools highlighter, but now somehow only
        /// used for the devtools accessibility inspector.
        const DEVTOOLS_HIGHLIGHTED = 1u64 << 45;
        /// Used for the devtools style editor. Probably should go away.
        const STYLEEDITOR_TRANSITIONING = 1u64 << 46;
        /// For :-moz-value-empty (to show widgets like the reveal password
        /// button or the clear button).
        const VALUE_EMPTY = 1u64 << 47;
        /// For :-moz-revealed.
        const REVEALED = 1u64 << 48;

        /// Some convenience unions.
        const DIR_STATES = Self::LTR.bits | Self::RTL.bits;

        const DIR_ATTR_STATES = Self::HAS_DIR_ATTR.bits |
                                Self::HAS_DIR_ATTR_LTR.bits |
                                Self::HAS_DIR_ATTR_RTL.bits |
                                Self::HAS_DIR_ATTR_LIKE_AUTO.bits;

        const DISABLED_STATES = Self::DISABLED.bits | Self::ENABLED.bits;

        const REQUIRED_STATES = Self::REQUIRED.bits | Self::OPTIONAL_.bits;

        /// Event states that can be added and removed through
        /// Element::{Add,Remove}ManuallyManagedStates.
        ///
        /// Take care when manually managing state bits.  You are responsible
        /// for setting or clearing the bit when an Element is added or removed
        /// from a document (e.g. in BindToTree and UnbindFromTree), if that is
        /// an appropriate thing to do for your state bit.
        const MANUALLY_MANAGED_STATES = Self::AUTOFILL.bits | Self::AUTOFILL_PREVIEW.bits;

        /// Event states that are managed externally to an element (by the
        /// EventStateManager, or by other code).  As opposed to those in
        /// INTRINSIC_STATES, which are are computed by the element itself
        /// and returned from Element::IntrinsicState.
        const EXTERNALLY_MANAGED_STATES =
            Self::MANUALLY_MANAGED_STATES.bits |
            Self::DIR_ATTR_STATES.bits |
            Self::DISABLED_STATES.bits |
            Self::REQUIRED_STATES.bits |
            Self::ACTIVE.bits |
            Self::DEFINED.bits |
            Self::DRAGOVER.bits |
            Self::FOCUS.bits |
            Self::FOCUSRING.bits |
            Self::FOCUS_WITHIN.bits |
            Self::FULLSCREEN.bits |
            Self::HOVER.bits |
            Self::URLTARGET.bits |
            Self::MODAL_DIALOG.bits |
            Self::INERT.bits |
            Self::TOPMOST_MODAL.bits |
            Self::REVEALED.bits;

        const INTRINSIC_STATES = !Self::EXTERNALLY_MANAGED_STATES.bits;
    }
}

bitflags! {
    /// Event-based document states.
    #[repr(C)]
    pub struct DocumentState: u64 {
        /// Window activation status
        const WINDOW_INACTIVE = 1 << 0;
        /// RTL locale: specific to the XUL localedir attribute
        const RTL_LOCALE = 1 << 1;
        /// LTR locale: specific to the XUL localedir attribute
        const LTR_LOCALE = 1 << 2;
        /// LWTheme status
        const LWTHEME = 1 << 3;

        const ALL_LOCALEDIR_BITS = Self::LTR_LOCALE.bits | Self::RTL_LOCALE.bits;
    }
}

malloc_size_of_is_0!(ElementState, DocumentState);
