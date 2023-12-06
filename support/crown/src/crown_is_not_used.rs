/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_lint::LintStore;
use rustc_session::declare_lint;

declare_lint! {
    CROWN_IS_NOT_USED,
    Deny,
    "Issues a rustc warning if crown is not used in compilation"
}

pub fn register(lint_store: &mut LintStore) {
    lint_store.register_lints(&[&CROWN_IS_NOT_USED]);
}
