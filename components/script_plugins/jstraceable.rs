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
        item2.attrs.push(quote_attr!(cx, #[repr(C)]));
        item2.attrs.push(quote_attr!(cx, #[derive(JSTraceable)]));
        item2.attrs.push(quote_attr!(cx, #[derive(HeapSizeOf)]));
        item2.attrs.push(quote_attr!(cx, #[derive(DenyPublicFields)]));
        item2.attrs.push(quote_attr!(cx, #[derive(DomObject)]));
        Annotatable::Item(P(item2))
    } else {
        cx.span_err(sp, "#[dom_struct] applied to something other than a struct");
        anno
    }
}
