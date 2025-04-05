/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct TestNS(());
