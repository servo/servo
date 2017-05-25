/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
 * This file contains a helper macro includes all supported non-tree-structural
 * pseudo-classes.
 *

 * FIXME: Find a way to autogenerate this file.
 *
 * Expected usage is as follows:
 * ```
 * macro_rules! pseudo_class_macro{
 *     (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
 *      string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
 *         // do stuff
 *     }
 * }
 * apply_non_ts_list!(pseudo_class_macro)
 * ```
 *
 * The string variables will be applied to pseudoclasses that are of the form
 * of a function with a string argument.
 *
 * Pending pseudo-classes:
 *
 *  :-moz-lwtheme, :-moz-lwtheme-brighttext, :-moz-lwtheme-darktext,
 *  :-moz-window-inactive.
 *
 *  :scope -> <style scoped>, pending discussion.
 *
 * This follows the order defined in layout/style/nsCSSPseudoClassList.h when
 * possible.
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
            bare: [
                ("unresolved", Unresolved, unresolved, IN_UNRESOLVED_STATE, _),
                ("-moz-table-border-nonzero", MozTableBorderNonzero, mozTableBorderNonzero, _, PSEUDO_CLASS_INTERNAL),
                ("-moz-browser-frame", MozBrowserFrame, mozBrowserFrame, _, PSEUDO_CLASS_INTERNAL),
                ("link", Link, link, IN_UNVISITED_STATE, _),
                ("any-link", AnyLink, anyLink, IN_VISITED_OR_UNVISITED_STATE, _),
                ("visited", Visited, visited, IN_VISITED_STATE, _),
                ("active", Active, active, IN_ACTIVE_STATE, _),
                ("checked", Checked, checked, IN_CHECKED_STATE, _),
                ("disabled", Disabled, disabled, IN_DISABLED_STATE, _),
                ("enabled", Enabled, enabled, IN_ENABLED_STATE, _),
                ("focus", Focus, focus, IN_FOCUS_STATE, _),
                ("focus-within", FocusWithin, focusWithin, IN_FOCUS_WITHIN_STATE, _),
                ("hover", Hover, hover, IN_HOVER_STATE, _),
                ("-moz-drag-over", MozDragOver, mozDragOver, IN_DRAGOVER_STATE, _),
                ("target", Target, target, IN_TARGET_STATE, _),
                ("indeterminate", Indeterminate, indeterminate, IN_INDETERMINATE_STATE, _),
                ("-moz-devtools-highlighted", MozDevtoolsHighlighted, mozDevtoolsHighlighted, IN_DEVTOOLS_HIGHLIGHTED_STATE, _),
                ("-moz-styleeditor-transitioning", MozStyleeditorTransitioning, mozStyleeditorTransitioning, IN_STYLEEDITOR_TRANSITIONING_STATE, _),
                // TODO(emilio): Needs pref check for
                // full-screen-api.unprefix.enabled!
                ("fullscreen", Fullscreen, fullscreen, IN_FULLSCREEN_STATE, _),
                // TODO(emilio): This is inconsistently named (the capital R).
                ("-moz-focusring", MozFocusRing, mozFocusRing, IN_FOCUSRING_STATE, _),
                ("-moz-broken", MozBroken, mozBroken, IN_BROKEN_STATE, _),
                ("-moz-loading", MozLoading, mozLoading, IN_LOADING_STATE, _),
                ("-moz-suppressed", MozSuppressed, mozSuppressed, IN_SUPPRESSED_STATE, PSEUDO_CLASS_INTERNAL),
                ("-moz-autofill", MozAutofill, mozAutofill, IN_AUTOFILL_STATE, PSEUDO_CLASS_INTERNAL),
                ("-moz-autofill-preview", MozAutofillPreview, mozAutofillPreview, IN_AUTOFILL_PREVIEW_STATE, PSEUDO_CLASS_INTERNAL),

                ("-moz-handler-clicktoplay", MozHandlerClickToPlay, mozHandlerClickToPlay, IN_HANDLER_CLICK_TO_PLAY_STATE, PSEUDO_CLASS_INTERNAL),
                ("-moz-handler-vulnerable-updatable", MozHandlerVulnerableUpdatable, mozHandlerVulnerableUpdatable, IN_HANDLER_VULNERABLE_UPDATABLE_STATE, PSEUDO_CLASS_INTERNAL),
                ("-moz-handler-vulnerable-no-update", MozHandlerVulnerableNoUpdate, mozHandlerVulnerableNoUpdate, IN_HANDLER_VULNERABLE_NO_UPDATE_STATE, PSEUDO_CLASS_INTERNAL),

                ("-moz-handler-disabled", MozHandlerDisabled, mozHandlerDisabled, IN_HANDLER_DISABLED_STATE, PSEUDO_CLASS_INTERNAL),
                ("-moz-handler-blocked", MozHandlerBlocked, mozHandlerBlocked, IN_HANDLER_BLOCKED_STATE, PSEUDO_CLASS_INTERNAL),
                ("-moz-handler-crashed", MozHandlerCrashed, mozHandlerCrashed, IN_HANDLER_CRASHED_STATE, PSEUDO_CLASS_INTERNAL),
                ("-moz-math-increment-script-level", MozMathIncrementScriptLevel, mozMathIncrementScriptLevel, IN_INCREMENT_SCRIPT_LEVEL_STATE, _),

                ("required", Required, required, IN_REQUIRED_STATE, _),
                ("optional", Optional, optional, IN_OPTIONAL_STATE, _),
                ("valid", Valid, valid, IN_VALID_STATE, _),
                ("invalid", Invalid, invalid, IN_INVALID_STATE, _),
                ("in-range", InRange, inRange, IN_INRANGE_STATE, _),
                ("out-of-range", OutOfRange, outOfRange, IN_OUTOFRANGE_STATE, _),
                ("default", Default, defaultPseudo, IN_DEFAULT_STATE, _),
                ("placeholder-shown", PlaceholderShown, placeholderShown, IN_PLACEHOLDER_SHOWN_STATE, _),
                ("-moz-read-only", MozReadOnly, mozReadOnly, IN_MOZ_READONLY_STATE, _),
                ("-moz-read-write", MozReadWrite, mozReadWrite, IN_MOZ_READWRITE_STATE, _),
                ("-moz-submit-invalid", MozSubmitInvalid, mozSubmitInvalid, IN_MOZ_SUBMITINVALID_STATE, _),
                ("-moz-ui-valid", MozUIValid, mozUIValid, IN_MOZ_UI_VALID_STATE, _),
                ("-moz-ui-invalid", MozUIInvalid, mozUIInvalid, IN_MOZ_UI_INVALID_STATE, _),
                ("-moz-meter-optimum", MozMeterOptimum, mozMeterOptimum, IN_OPTIMUM_STATE, _),
                ("-moz-meter-sub-optimum", MozMeterSubOptimum, mozMeterSubOptimum, IN_SUB_OPTIMUM_STATE, _),
                ("-moz-meter-sub-sub-optimum", MozMeterSubSubOptimum, mozMeterSubSubOptimum, IN_SUB_SUB_OPTIMUM_STATE, _),

                ("-moz-user-disabled", MozUserDisabled, mozUserDisabled, IN_USER_DISABLED_STATE, PSEUDO_CLASS_INTERNAL),

                ("-moz-first-node", MozFirstNode, firstNode, _, _),
                ("-moz-last-node", MozLastNode, lastNode, _, _),
                ("-moz-only-whitespace", MozOnlyWhitespace, mozOnlyWhitespace, _, _),
                ("-moz-native-anonymous", MozNativeAnonymous, mozNativeAnonymous, _, PSEUDO_CLASS_INTERNAL),
                ("-moz-is-html", MozIsHTML, mozIsHTML, _, _),
                ("-moz-placeholder", MozPlaceholder, mozPlaceholder, _, _),
            ],
            string: [
                ("-moz-system-metric", MozSystemMetric, mozSystemMetric, _, PSEUDO_CLASS_INTERNAL),
                ("-moz-locale-dir", MozLocaleDir, mozLocaleDir, _, PSEUDO_CLASS_INTERNAL),
                ("-moz-empty-except-children-with-localname", MozEmptyExceptChildrenWithLocalname,
                 mozEmptyExceptChildrenWithLocalname, _, PSEUDO_CLASS_INTERNAL),
                ("dir", Dir, dir, _, _),
                ("lang", Lang, lang, _, _),
            ]
        }
    }
}
