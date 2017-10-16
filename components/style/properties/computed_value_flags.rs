/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Misc information about a given computed style.

bitflags! {
    /// Misc information about a given computed style.
    ///
    /// All flags are currently inherited for text, pseudo elements, and
    /// anonymous boxes, see StyleBuilder::for_inheritance and its callsites.
    /// If we ever want to add some flags that shouldn't inherit for them,
    /// we might want to add a function to handle this.
    pub flags ComputedValueFlags: u16 {
        /// Whether the style or any of the ancestors has a text-decoration-line
        /// property that should get propagated to descendants.
        ///
        /// text-decoration-line is a reset property, but gets propagated in the
        /// frame/box tree.
        const HAS_TEXT_DECORATION_LINES = 1 << 0,

        /// Whether line break inside should be suppressed.
        ///
        /// If this flag is set, the line should not be broken inside,
        /// which means inlines act as if nowrap is set, <br> element is
        /// suppressed, and blocks are inlinized.
        ///
        /// This bit is propagated to all children of line participants.
        /// It is currently used by ruby to make its content unbreakable.
        const SHOULD_SUPPRESS_LINEBREAK = 1 << 1,

        /// A flag used to mark text that that has text-combine-upright.
        ///
        /// This is used from Gecko's layout engine.
        const IS_TEXT_COMBINED = 1 << 2,

        /// A flag used to mark styles under a relevant link that is also
        /// visited.
        const IS_RELEVANT_LINK_VISITED = 1 << 3,

        /// A flag used to mark styles which are a pseudo-element or under one.
        const IS_IN_PSEUDO_ELEMENT_SUBTREE = 1 << 4,

        /// A flag used to mark styles which are in a display: none subtree, or
        /// under one.
        const IS_IN_DISPLAY_NONE_SUBTREE = 1 << 5,

        /// Whether this style inherits the `display` property.
        ///
        /// This is important because it may affect our optimizations to avoid
        /// computing the style of pseudo-elements, given whether the
        /// pseudo-element is generated depends on the `display` value.
        const INHERITS_DISPLAY = 1 << 6,

        /// Whether this style inherits the `content` property.
        ///
        /// Important because of the same reason.
        const INHERITS_CONTENT = 1 << 7,

        /// Whether the child explicitly inherits any reset property.
        const INHERITS_RESET_STYLE = 1 << 8,

        /// A flag to mark a style which is a visited style.
        const IS_STYLE_IF_VISITED = 1 << 9,

        /// Whether the style or any of the ancestors has a multicol style.
        ///
        /// Only used in Servo.
        const CAN_BE_FRAGMENTED = 1 << 10,
    }
}

impl ComputedValueFlags {
    /// Returns the flags that are inherited.
    #[inline]
    pub fn inherited(self) -> Self {
        self & !(INHERITS_DISPLAY | INHERITS_CONTENT | INHERITS_RESET_STYLE)
    }
}
