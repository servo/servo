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
 * $gecko_type can be either "_" or an ident in Gecko's CSSPseudoClassType.
 * $state can be either "_" or an expression of type ElementState.
 * $flags can be either "_" or an expression of type NonTSPseudoClassFlag,
 * see selector_parser.rs for more details.
 */

macro_rules! apply_non_ts_list {
    ($apply_macro:ident) => {
        $apply_macro! {
            bare: [
                ("any-link", AnyLink, anyLink, _, _),
                ("link", Link, link, _, _),
                ("visited", Visited, visited, _, _),
                ("active", Active, active, IN_ACTIVE_STATE, _),
                ("focus", Focus, focus, IN_FOCUS_STATE, _),
                ("fullscreen", Fullscreen, fullscreen, IN_FULLSCREEN_STATE, _),
                ("hover", Hover, hover, IN_HOVER_STATE, _),
                ("enabled", Enabled, enabled, IN_ENABLED_STATE, _),
                ("disabled", Disabled, disabled, IN_DISABLED_STATE, _),
                ("checked", Checked, checked, IN_CHECKED_STATE, _),
                ("indeterminate", Indeterminate, indeterminate, IN_INDETERMINATE_STATE, _),
                ("placeholder-shown", PlaceholderShown, placeholderShown, IN_PLACEHOLDER_SHOWN_STATE, _),
                ("target", Target, target, IN_TARGET_STATE, _),
                ("valid", Valid, valid, IN_VALID_STATE, _),
                ("invalid", Invalid, invalid, IN_INVALID_STATE, _),
                ("-moz-ui-valid", MozUIValid, mozUIValid, IN_MOZ_UI_VALID_STATE, _),
                ("read-write", ReadWrite, _, IN_READ_WRITE_STATE, _),
                ("read-only", ReadOnly, _, IN_READ_WRITE_STATE, _),

                ("-moz-browser-frame", MozBrowserFrame, mozBrowserFrame, _, PSEUDO_CLASS_INTERNAL),
                ("-moz-table-border-nonzero", MozTableBorderNonzero, mozTableBorderNonzero, _, PSEUDO_CLASS_INTERNAL),
            ],
            string: [
                ("-moz-system-metric", MozSystemMetric, mozSystemMetric, _, PSEUDO_CLASS_INTERNAL),
            ]
        }
    }
}
