/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syntax::ext::base::ExtCtxt;
use syntax::codemap::Span;
use syntax::ptr::P;
use syntax::ast::{Item, MetaItem};
use syntax::ast;
use utils::match_ty_unwrap;


pub fn expand_reflector(cx: &mut ExtCtxt, span: Span, _: &MetaItem, item: &Item, push: |P<Item>|) {

    if let ast::ItemStruct(ref def, _) = item.node {
        let struct_name = item.ident;
        match def.fields.iter().find(|f| match_ty_unwrap(&*f.node.ty, &["dom", "bindings", "utils", "Reflector"]).is_some()) {
            // If it has a field that is a Reflector, use that
            Some(f) => {
                let field_name = f.node.ident();
                let impl_item = quote_item!(cx,
                    impl ::dom::bindings::utils::Reflectable for $struct_name {
                        fn reflector<'a>(&'a self) -> &'a ::dom::bindings::utils::Reflector {
                            &self.$field_name
                        }
                    }
                );
                impl_item.map(|it| push(it))
            },
            // Or just call it on the first field (supertype).
            // TODO: Write a lint to ensure that this first field is indeed a #[dom_struct],
            // and the only such field in the struct definition (including reflectors)
            // Unfortunately we can't do it here itself because a def_map (from middle) is not available
            // at expansion time
            None => {
                let field_name = def.fields[0].node.ident();
                let impl_item = quote_item!(cx,
                    impl ::dom::bindings::utils::Reflectable for $struct_name {
                        fn reflector<'a>(&'a self) -> &'a ::dom::bindings::utils::Reflector {
                            self.$field_name.reflector()
                        }
                    }
                );
                impl_item.map(|it| push(it))
            }
        };
    } else {
        cx.span_bug(span, "#[dom_struct] seems to have been applied to a non-struct");
    }
}
