/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::error::Error;
use syntax;
use syntax::ast::{TokenTree, ExprLit, LitStr, Expr};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacEager, DummyResult};
use syntax::ext::build::AstBuilder;
use syntax::fold::Folder;
use syntax::parse;
use syntax::parse::token::InternedString;
use url::{Url, Host, RelativeSchemeData, SchemeData};

pub fn expand_url(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(), tts.to_vec());
    let query_expr = cx.expander().fold_expr(parser.parse_expr());

    // Ensure a str literal was passed to the macro
    let query = match parse_str_lit(&query_expr) {
        Some(query) => query,
        None => {
            cx.span_err(query_expr.span, "'url!' expected string literal");
            return DummyResult::any(sp)
        },
    };

    // Parse the str literal
    let Url { scheme, scheme_data, query, fragment } = match Url::parse(&query) {
        Ok(url) => url,
        Err(error) => {
            cx.span_err(query_expr.span, error.description());
            return DummyResult::any(sp)
        }
    };

    let scheme_data_expr = cx.expr_scheme_data(sp, scheme_data);
    let query_expr = cx.expr_option_string(sp, query);
    let fragment_expr = cx.expr_option_string(sp, fragment);

    let url_expr = quote_expr!(cx, {
        ::url::Url {
            scheme: $scheme.to_owned(),
            scheme_data: $scheme_data_expr,
            query: $query_expr,
            fragment: $fragment_expr,
        }
    });

    MacEager::expr(url_expr)
}

fn parse_str_lit(e: &Expr) -> Option<InternedString> {
    if let ExprLit(ref lit) = e.node {
        if let LitStr(ref s, _) = lit.node {
            return Some(s.clone());
        }
    }
    None
}

trait ExtCtxtHelpers {
    fn expr_scheme_data(&self, sp: Span, scheme_data: SchemeData) -> syntax::ptr::P<Expr>;
    fn expr_option_string(&self, sp: Span, string: Option<String>) -> syntax::ptr::P<Expr>;
    fn expr_option_u16(&self, sp: Span, unsigned: Option<u16>) -> syntax::ptr::P<Expr>;
    fn expr_host(&self, sp: Span, host: Host) -> syntax::ptr::P<Expr>;
    fn expr_slice_u16(&self, sp: Span, unsigned: &[u16]) -> syntax::ptr::P<Expr>;
    fn expr_vec_string(&self, sp: Span, strings: Vec<String>) -> syntax::ptr::P<Expr>;
}

impl<'a> ExtCtxtHelpers for ExtCtxt<'a> {
    fn expr_scheme_data(&self, sp: Span, scheme_data: SchemeData) -> syntax::ptr::P<Expr> {
        match scheme_data {
            SchemeData::Relative(
                RelativeSchemeData { username, password, host, port, default_port, path }) =>
            {
                let password_expr = self.expr_option_string(sp, password);
                let host_expr = self.expr_host(sp, host);
                let port_expr = self.expr_option_u16(sp, port);
                let default_port_expr = self.expr_option_u16(sp, default_port);
                let path_expr = self.expr_vec_string(sp, path);

                quote_expr!(self,
                            ::url::SchemeData::Relative(
                                ::url::RelativeSchemeData {
                                    username: $username.to_owned(),
                                    password: $password_expr,
                                    host: $host_expr,
                                    port: $port_expr,
                                    default_port: $default_port_expr,
                                    path: $path_expr.to_owned(),
                                }
                            ))
            },
            SchemeData::NonRelative(ref scheme_data) => {
                quote_expr!(self, ::url::SchemeData::NonRelative($scheme_data.to_owned()))
            },
        }
    }

    fn expr_option_string(&self, sp: Span, string: Option<String>) -> syntax::ptr::P<Expr> {
        match string {
            Some(string) => quote_expr!(self, Some($string.to_owned())),
            None => self.expr_none(sp),
        }
    }

    fn expr_option_u16(&self, sp: Span, unsigned: Option<u16>) -> syntax::ptr::P<Expr> {
        match unsigned {
            Some(unsigned) => quote_expr!(self, Some($unsigned)),
            None => self.expr_none(sp),
        }
    }

    fn expr_host(&self, sp: Span, host: Host) -> syntax::ptr::P<Expr> {
        match host {
            Host::Domain(domain) => quote_expr!(self, ::url::Host::Domain(String::from($domain))),
            Host::Ipv6(address) => {
                let pieces_expr = self.expr_slice_u16(sp, &address.pieces);
                quote_expr!(self,
                            ::url::Host::Ipv6(
                                ::url::Ipv6Address {
                                    pieces: $pieces_expr.to_owned()
                                }
                            ))
            },
        }
    }

    fn expr_slice_u16(&self, sp: Span, unsigned: &[u16]) -> syntax::ptr::P<Expr> {
        let unsigned = unsigned.iter().map(|p| quote_expr!(self, $p)).collect();
        self.expr_vec_slice(sp, unsigned)
    }

    fn expr_vec_string(&self, sp: Span, strings: Vec<String>) -> syntax::ptr::P<Expr> {
        let strings = strings.iter().map(|p| quote_expr!(self, $p.to_owned())).collect();
        self.expr_vec(sp, strings)
    }
}
