/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syntax::ast;
use syntax::ast::MetaItem;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use utils::match_ty_unwrap;


pub fn expand_reflector(cx: &mut ExtCtxt, span: Span, _: &MetaItem, annotatable: &Annotatable,
                        push: &mut FnMut(Annotatable)) {
    if let &Annotatable::Item(ref item) = annotatable {
        if let ast::ItemStruct(ref def, _) = item.node {
            let struct_name = item.ident;
            // This path has to be hardcoded, unfortunately, since we can't resolve paths at expansion time
            match def.fields.iter().find(
                    |f| match_ty_unwrap(&*f.node.ty, &["dom", "bindings", "utils", "Reflector"]).is_some()) {
                // If it has a field that is a Reflector, use that
                Some(f) => {
                    let field_name = f.node.ident();
                    let impl_item = quote_item!(cx,
                        impl ::dom::bindings::utils::Reflectable for $struct_name {
                            fn reflector<'a>(&'a self) -> &'a ::dom::bindings::utils::Reflector {
                                &self.$field_name
                            }
                            fn init_reflector(&mut self, obj: *mut ::js::jsapi::JSObject) {
                                self.$field_name.set_jsobject(obj);
                            }
                        }
                    );
                    impl_item.map(|it| push(Annotatable::Item(it)))
                },
                // Or just call it on the first field (supertype).
                None => {
                    let field_name = def.fields[0].node.ident();
                    let impl_item = quote_item!(cx,
                        impl ::dom::bindings::utils::Reflectable for $struct_name {
                            fn reflector<'a>(&'a self) -> &'a ::dom::bindings::utils::Reflector {
                                self.$field_name.reflector()
                            }
                            fn init_reflector(&mut self, obj: *mut ::js::jsapi::JSObject) {
                                self.$field_name.init_reflector(obj);
                            }
                        }
                    );
                    impl_item.map(|it| push(Annotatable::Item(it)))
                }
            };
        } else {
            cx.span_err(span, "#[dom_struct] seems to have been applied to a non-struct");
        }
    }
}
