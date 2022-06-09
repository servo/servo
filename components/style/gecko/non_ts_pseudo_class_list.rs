/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/*
 * This file contains a helper macro includes all supported non-tree-structural
 * pseudo-classes.
 *
 * FIXME: Find a way to autogenerate this file.
 *
 * Expected usage is as follows:
 * ```
 * macro_rules! pseudo_class_macro{
 *     ([$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*]) => {
 *         // do stuff
 *     }
 * }
 * apply_non_ts_list!(pseudo_class_macro)
 * ```
 *
 * $gecko_type can be either "_" or an ident in Gecko's CSSPseudoClassType.
 * $state can be either "_" or an expression of type ElementState.  If present,
 *        the semantics are that the pseudo-class matches if any of the bits in
 *        $state are set on the element.
 * $flags can be either "_" or an expression of type NonTSPseudoClassFlag,
 * see selector_parser.rs for more details.
 */

macro_rules! apply_non_ts_list {
    ($apply_macro:ident) => {
        $apply_macro! {
            [
                ("-moz-table-border-nonzero", MozTableBorderNonzero, _, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-browser-frame", MozBrowserFrame, _, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
                ("-moz-select-list-box", MozSelectListBox, _, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("link", Link, UNVISITED, _),
                ("any-link", AnyLink, VISITED_OR_UNVISITED, _),
                ("visited", Visited, VISITED, _),
                ("active", Active, ACTIVE, _),
                ("autofill", Autofill, AUTOFILL, _),
                ("checked", Checked, CHECKED, _),
                ("defined", Defined, DEFINED, _),
                ("disabled", Disabled, DISABLED, _),
                ("enabled", Enabled, ENABLED, _),
                ("focus", Focus, FOCUS, _),
                ("focus-within", FocusWithin, FOCUS_WITHIN, _),
                ("focus-visible", FocusVisible, FOCUSRING, _),
                ("hover", Hover, HOVER, _),
                ("-moz-drag-over", MozDragOver, DRAGOVER, _),
                ("target", Target, URLTARGET, _),
                ("indeterminate", Indeterminate, INDETERMINATE, _),
                ("-moz-inert", MozInert, INERT, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-devtools-highlighted", MozDevtoolsHighlighted, DEVTOOLS_HIGHLIGHTED, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-styleeditor-transitioning", MozStyleeditorTransitioning, STYLEEDITOR_TRANSITIONING, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("fullscreen", Fullscreen, FULLSCREEN, _),
                ("modal", Modal, MODAL_DIALOG, _),
                ("-moz-topmost-modal", MozTopmostModal, TOPMOST_MODAL, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-broken", MozBroken, BROKEN, _),
                ("-moz-loading", MozLoading, LOADING, _),
                ("-moz-has-dir-attr", MozHasDirAttr, HAS_DIR_ATTR, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-dir-attr-ltr", MozDirAttrLTR, HAS_DIR_ATTR_LTR, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-dir-attr-rtl", MozDirAttrRTL, HAS_DIR_ATTR_RTL, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-dir-attr-like-auto", MozDirAttrLikeAuto, HAS_DIR_ATTR_LIKE_AUTO, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),

                ("-moz-autofill-preview", MozAutofillPreview, AUTOFILL_PREVIEW, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
                ("-moz-value-empty", MozValueEmpty, VALUE_EMPTY, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-revealed", MozRevealed, REVEALED, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),

                ("-moz-math-increment-script-level", MozMathIncrementScriptLevel, INCREMENT_SCRIPT_LEVEL, _),

                ("required", Required, REQUIRED, _),
                ("optional", Optional, OPTIONAL_, _),
                ("valid", Valid, VALID, _),
                ("invalid", Invalid, INVALID, _),
                ("in-range", InRange, INRANGE, _),
                ("out-of-range", OutOfRange, OUTOFRANGE, _),
                ("default", Default, DEFAULT, _),
                ("placeholder-shown", PlaceholderShown, PLACEHOLDER_SHOWN, _),
                ("read-only", ReadOnly, READONLY, _),
                ("read-write", ReadWrite, READWRITE, _),
                ("user-valid", UserValid, USER_VALID, _),
                ("user-invalid", UserInvalid, USER_INVALID, _),
                ("-moz-meter-optimum", MozMeterOptimum, OPTIMUM, _),
                ("-moz-meter-sub-optimum", MozMeterSubOptimum, SUB_OPTIMUM, _),
                ("-moz-meter-sub-sub-optimum", MozMeterSubSubOptimum, SUB_SUB_OPTIMUM, _),

                ("-moz-first-node", MozFirstNode, _, _),
                ("-moz-last-node", MozLastNode, _, _),
                ("-moz-only-whitespace", MozOnlyWhitespace, _, _),
                ("-moz-native-anonymous", MozNativeAnonymous, _, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-use-shadow-tree-root", MozUseShadowTreeRoot, _, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-is-html", MozIsHTML, _, _),
                ("-moz-placeholder", MozPlaceholder, _, _),
                ("-moz-lwtheme", MozLWTheme, _, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
                ("-moz-window-inactive", MozWindowInactive, _, _),
            ]
        }
    }
}
