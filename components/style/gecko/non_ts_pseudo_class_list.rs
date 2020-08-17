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
                ("link", Link, IN_UNVISITED_STATE, _),
                ("any-link", AnyLink, IN_VISITED_OR_UNVISITED_STATE, _),
                ("visited", Visited, IN_VISITED_STATE, _),
                ("active", Active, IN_ACTIVE_STATE, _),
                ("checked", Checked, IN_CHECKED_STATE, _),
                ("defined", Defined, IN_DEFINED_STATE, _),
                ("disabled", Disabled, IN_DISABLED_STATE, _),
                ("enabled", Enabled, IN_ENABLED_STATE, _),
                ("focus", Focus, IN_FOCUS_STATE, _),
                ("focus-within", FocusWithin, IN_FOCUS_WITHIN_STATE, _),
                ("focus-visible", FocusVisible, IN_FOCUS_VISIBLE_STATE, _),
                ("hover", Hover, IN_HOVER_STATE, _),
                ("-moz-drag-over", MozDragOver, IN_DRAGOVER_STATE, _),
                ("target", Target, IN_TARGET_STATE, _),
                ("indeterminate", Indeterminate, IN_INDETERMINATE_STATE, _),
                ("-moz-inert", MozInert, IN_MOZINERT_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-devtools-highlighted", MozDevtoolsHighlighted, IN_DEVTOOLS_HIGHLIGHTED_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-styleeditor-transitioning", MozStyleeditorTransitioning, IN_STYLEEDITOR_TRANSITIONING_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("fullscreen", Fullscreen, IN_FULLSCREEN_STATE, _),
                ("-moz-modal-dialog", MozModalDialog, IN_MODAL_DIALOG_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-topmost-modal-dialog", MozTopmostModalDialog, IN_TOPMOST_MODAL_DIALOG_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                // TODO(emilio): This is inconsistently named (the capital R).
                ("-moz-focusring", MozFocusRing, IN_FOCUSRING_STATE, _),
                ("-moz-broken", MozBroken, IN_BROKEN_STATE, _),
                ("-moz-loading", MozLoading, IN_LOADING_STATE, _),
                ("-moz-suppressed", MozSuppressed, IN_SUPPRESSED_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
                ("-moz-has-dir-attr", MozHasDirAttr, IN_HAS_DIR_ATTR_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-dir-attr-ltr", MozDirAttrLTR, IN_HAS_DIR_ATTR_LTR_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-dir-attr-rtl", MozDirAttrRTL, IN_HAS_DIR_ATTR_RTL_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-dir-attr-like-auto", MozDirAttrLikeAuto, IN_HAS_DIR_ATTR_LIKE_AUTO_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-autofill", MozAutofill, IN_AUTOFILL_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
                ("-moz-autofill-preview", MozAutofillPreview, IN_AUTOFILL_PREVIEW_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),

                ("-moz-handler-clicktoplay", MozHandlerClickToPlay, IN_HANDLER_CLICK_TO_PLAY_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
                ("-moz-handler-vulnerable-updatable", MozHandlerVulnerableUpdatable, IN_HANDLER_VULNERABLE_UPDATABLE_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
                ("-moz-handler-vulnerable-no-update", MozHandlerVulnerableNoUpdate, IN_HANDLER_VULNERABLE_NO_UPDATE_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),

                ("-moz-handler-disabled", MozHandlerDisabled, IN_HANDLER_DISABLED_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
                ("-moz-handler-blocked", MozHandlerBlocked, IN_HANDLER_BLOCKED_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
                ("-moz-handler-crashed", MozHandlerCrashed, IN_HANDLER_CRASHED_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
                ("-moz-math-increment-script-level", MozMathIncrementScriptLevel, IN_INCREMENT_SCRIPT_LEVEL_STATE, _),

                ("required", Required, IN_REQUIRED_STATE, _),
                ("optional", Optional, IN_OPTIONAL_STATE, _),
                ("valid", Valid, IN_VALID_STATE, _),
                ("invalid", Invalid, IN_INVALID_STATE, _),
                ("in-range", InRange, IN_INRANGE_STATE, _),
                ("out-of-range", OutOfRange, IN_OUTOFRANGE_STATE, _),
                ("default", Default, IN_DEFAULT_STATE, _),
                ("placeholder-shown", PlaceholderShown, IN_PLACEHOLDER_SHOWN_STATE, _),
                ("read-only", ReadOnly, IN_READONLY_STATE, _),
                ("read-write", ReadWrite, IN_READWRITE_STATE, _),
                ("-moz-submit-invalid", MozSubmitInvalid, IN_MOZ_SUBMITINVALID_STATE, _),
                ("-moz-ui-valid", MozUIValid, IN_MOZ_UI_VALID_STATE, _),
                ("-moz-ui-invalid", MozUIInvalid, IN_MOZ_UI_INVALID_STATE, _),
                ("-moz-meter-optimum", MozMeterOptimum, IN_OPTIMUM_STATE, _),
                ("-moz-meter-sub-optimum", MozMeterSubOptimum, IN_SUB_OPTIMUM_STATE, _),
                ("-moz-meter-sub-sub-optimum", MozMeterSubSubOptimum, IN_SUB_SUB_OPTIMUM_STATE, _),

                ("-moz-user-disabled", MozUserDisabled, IN_USER_DISABLED_STATE, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),

                ("-moz-first-node", MozFirstNode, _, _),
                ("-moz-last-node", MozLastNode, _, _),
                ("-moz-only-whitespace", MozOnlyWhitespace, _, _),
                ("-moz-native-anonymous", MozNativeAnonymous, _, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-native-anonymous-no-specificity", MozNativeAnonymousNoSpecificity, _, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-use-shadow-tree-root", MozUseShadowTreeRoot, _, PSEUDO_CLASS_ENABLED_IN_UA_SHEETS),
                ("-moz-is-html", MozIsHTML, _, _),
                ("-moz-placeholder", MozPlaceholder, _, _),
                ("-moz-lwtheme", MozLWTheme, _, _),
                ("-moz-lwtheme-brighttext", MozLWThemeBrightText, _, _),
                ("-moz-lwtheme-darktext", MozLWThemeDarkText, _, _),
                ("-moz-window-inactive", MozWindowInactive, _, _),
            ]
        }
    }
}
