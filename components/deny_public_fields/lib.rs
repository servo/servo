/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate synstructure;

use std::str::FromStr;

decl_derive!([DenyPublicFields] => deny_public_fields_derive);

fn deny_public_fields_derive(s: synstructure::Structure) -> proc_macro::TokenStream {
    s.each(|binding| {
        if binding.ast().vis != syn::Visibility::Inherited {
            panic!("Field `{}` should not be public",
                binding.ast().ident.as_ref().unwrap_or(&binding.binding));
        }

        "".to_owned()
    });

    proc_macro::TokenStream::from_str("").unwrap()
}
