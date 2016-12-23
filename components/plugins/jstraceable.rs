/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syntax::ast::MetaItem;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;

pub fn expand_dom_struct(cx: &mut ExtCtxt, sp: Span, _: &MetaItem, anno: Annotatable) -> Annotatable {
    if let Annotatable::Item(item) = anno {
        let mut item2 = (*item).clone();
        item2.attrs.push(quote_attr!(cx, #[must_root]));
        item2.attrs.push(quote_attr!(cx, #[privatize]));
        item2.attrs.push(quote_attr!(cx, #[repr(C)]));
        item2.attrs.push(quote_attr!(cx, #[derive(JSTraceable)]));
        item2.attrs.push(quote_attr!(cx, #[derive(HeapSizeOf)]));

        // The following attributes are only for internal usage
        item2.attrs.push(quote_attr!(cx, #[_generate_reflector]));
        // #[dom_struct] gets consumed, so this lets us keep around a residue
        // Do NOT register a modifier/decorator on this attribute
        item2.attrs.push(quote_attr!(cx, #[_dom_struct_marker]));
        Annotatable::Item(P(item2))
    } else {
        cx.span_err(sp, "#[dom_struct] applied to something other than a struct");
        anno
    }
}
