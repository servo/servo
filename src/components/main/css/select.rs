/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::net::url::Url;
use url_from_str = std::net::url::from_str;
use core::cell::Cell;
use newcss::stylesheet::Stylesheet;
use newcss::select::SelectCtx;
use newcss::types::OriginUA;
use newcss::util::DataStream;

pub fn new_css_select_ctx() -> SelectCtx {
    let mut ctx = SelectCtx::new();
    ctx.append_sheet(html4_default_style(), OriginUA);
    ctx.append_sheet(servo_default_style(), OriginUA);
    return ctx;
}

fn html4_default_style() -> Stylesheet {
    Stylesheet::new(default_url("html4_style"),
                    style_stream(html4_default_style_str()))
}

fn servo_default_style() -> Stylesheet {
    Stylesheet::new(default_url("servo_style"),
                    style_stream(servo_default_style_str()))
}

fn default_url(name: &str) -> Url {
    result::unwrap(url_from_str(fmt!("http://%s", name)))
}

fn style_stream(style: &str) -> DataStream {
    let style = Cell(str::to_bytes(style));
    let d: DataStream = || if !style.is_empty() {
        Some(style.take())
    } else {
        None
    };
    return d;
}

fn html4_default_style_str() -> ~str {
~"
html, address,
blockquote,
body, dd, div,
dl, dt, fieldset, form,
frame, frameset,
h1, h2, h3, h4,
h5, h6, noframes,
ol, p, ul, center,
    dir, hr, menu, pre   { display: block; unicode-bidi: embed }
    li              { display: list-item }
    head            { display: none }
    table           { display: table }
    tr              { display: table-row }
    thead           { display: table-header-group }
    tbody           { display: table-row-group }
    tfoot           { display: table-footer-group }
    col             { display: table-column }
    colgroup        { display: table-column-group }
    td, th          { display: table-cell }
    caption         { display: table-caption }
    th              { font-weight: bolder; text-align: center }
    caption         { text-align: center }
    body            { margin: 8px }
    h1              { font-size: 2em; margin: .67em 0 }
    h2              { font-size: 1.5em; margin: .75em 0 }
    h3              { font-size: 1.17em; margin: .83em 0 }
h4, p,
blockquote, ul,
fieldset, form,
ol, dl, dir,
    menu            { margin: 1.12em 0 }
    h5              { font-size: .83em; margin: 1.5em 0 }
    h6              { font-size: .75em; margin: 1.67em 0 }
h1, h2, h3, h4,
h5, h6, b,
    strong          { font-weight: bolder }
    blockquote      { margin-left: 40px; margin-right: 40px }
i, cite, em,
    var, address    { font-style: italic }
pre, tt, code,
    kbd, samp       { font-family: monospace }
    pre             { white-space: pre }
button, textarea,
    input, select   { display: inline-block }
    big             { font-size: 1.17em }
    small, sub, sup { font-size: .83em }
    sub             { vertical-align: sub }
    sup             { vertical-align: super }
    table           { border-spacing: 2px; }
thead, tbody,
    tfoot           { vertical-align: middle }
    td, th, tr      { vertical-align: inherit }
    s, strike, del  { text-decoration: line-through }
    hr              { border: 1px inset }
ol, ul, dir,
    menu, dd        { margin-left: 40px }
    ol              { list-style-type: decimal }
ol ul, ul ol,
    ul ul, ol ol    { margin-top: 0; margin-bottom: 0 }
    u, ins          { text-decoration: underline }
    br:before       { content: \"\\A\"; white-space: pre-line }
center          { text-align: center }
:link, :visited { text-decoration: underline }
:focus          { outline: thin dotted invert }

/* Begin bidirectionality settings (do not change) */
BDO[DIR=\"ltr\"]  { direction: ltr; unicode-bidi: bidi-override }
BDO[DIR=\"rtl\"]  { direction: rtl; unicode-bidi: bidi-override }

*[DIR=\"ltr\"]    { direction: ltr; unicode-bidi: embed }
*[DIR=\"rtl\"]    { direction: rtl; unicode-bidi: embed }

@media print {
h1            { page-break-before: always }
  h1, h2, h3,
h4, h5, h6    { page-break-after: avoid }
ul, ol, dl    { page-break-before: avoid }
}

/* Servo additions */
:link           { color: blue }
script          { display: none }
style           { display: none }
"
}

fn servo_default_style_str() -> ~str {
    // libcss want's this to default to 2px..
    ~"* { border-width: 0px; }"
}
