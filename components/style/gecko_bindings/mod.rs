/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[allow(dead_code, improper_ctypes, non_camel_case_types)]
pub mod bindings;

// FIXME: We allow `improper_ctypes` (for now), because the lint doesn't allow
// foreign structs to have `PhantomData`. We should remove this once the lint
// ignores this case.

#[cfg(debug_assertions)]
#[allow(dead_code, improper_ctypes, non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod structs {
    include!("structs_debug.rs");
}

#[cfg(not(debug_assertions))]
#[allow(dead_code, improper_ctypes, non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod structs {
    include!("structs_release.rs");
}

pub mod sugar;
