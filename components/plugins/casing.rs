/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::codemap::Span;
use syntax::ast;
use syntax::ext::base;
use syntax::parse::token;

pub fn expand_lower<'cx>(cx: &'cx mut ExtCtxt, sp: Span, tts: &[ast::TokenTree])
                        -> Box<base::MacResult + 'cx> {
    expand_cased(cx, sp, tts, |s| { s.to_lowercase() })
}

pub fn expand_upper<'cx>(cx: &'cx mut ExtCtxt, sp: Span, tts: &[ast::TokenTree])
                        -> Box<base::MacResult + 'cx> {
    expand_cased(cx, sp, tts, |s| { s.to_uppercase() })
}

fn expand_cased<'cx, T>(cx: &'cx mut ExtCtxt, sp: Span, tts: &[ast::TokenTree], transform: T)
                        -> Box<base::MacResult + 'cx>
    where T: Fn(&str) -> String
{
    let es = match base::get_exprs_from_tts(cx, sp, tts) {
        Some(e) => e,
        None => return base::DummyResult::expr(sp)
    };

    let mut it = es.iter();
    let res = if let Some(expr) = it.next() {
        if let ast::ExprLit(ref lit) = expr.node {
            if let ast::LitStr(ref s, _) = lit.node {
                Some((s, lit.span))
            } else {
                cx.span_err(expr.span, "expected a string literal");
                None
            }
        } else {
            cx.span_err(expr.span, "expected a string literal");
            None
        }
    } else {
        cx.span_err(sp, "expected 1 argument, found 0");
        None
    };
    match (res, it.count()) {
        (Some((s, span)), 0) => {
            base::MacEager::expr(cx.expr_str(span, token::intern_and_get_ident(&transform(&s))))
        }
        (_, rest) => {
            if rest > 0 {
                cx.span_err(sp, &format!("expected 1 argument, found {}", rest + 1));
            }
            base::DummyResult::expr(sp)
        }
    }
}
