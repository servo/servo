/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Supported CSS properties and the cascade.

pub mod cascade;
pub mod declaration_block;

/// The CSS properties supported by the style system.
/// Generated from the properties.mako.rs template by build.rs
#[macro_use]
#[allow(unsafe_code)]
#[deny(missing_docs)]
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/properties.rs"));

    #[cfg(feature = "gecko")]
    #[allow(unsafe_code, missing_docs)]
    pub mod gecko {
        include!(concat!(env!("OUT_DIR"), "/gecko_properties.rs"));
    }
}

pub use self::cascade::*;
pub use self::declaration_block::*;
pub use self::generated::*;
