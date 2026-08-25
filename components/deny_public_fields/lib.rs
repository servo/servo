/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use synstructure::{self, decl_derive};

decl_derive!([DenyPublicFields] => deny_public_fields_derive);

fn deny_public_fields_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    s.each(|binding| {
        if binding.ast().vis != syn::Visibility::Inherited {
            panic!(
                "Field `{}` should not be public",
                binding.ast().ident.as_ref().unwrap_or(&binding.binding)
            );
        }

        proc_macro2::TokenStream::new()
    });

    proc_macro2::TokenStream::new()
}

#[test]
#[should_panic(expected = "Field `v1` should not be public")]
fn deny_public_fields_failing() {
    synstructure::test_derive! {
        deny_public_fields_derive {
            struct Foo {
                pub v1: i32,
                v2: i32
            }
        }
        expands to {}
    };
}

#[test]
fn deny_public_fields_ok() {
    synstructure::test_derive! {
        deny_public_fields_derive {
            struct Foo {
                v1: i32,
                v2: i32
            }
        }
        expands to {}
    };
}
