#[allow(dead_code, improper_ctypes, non_camel_case_types)]
pub mod bindings;
pub mod ptr;

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
