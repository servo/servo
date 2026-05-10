// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-white-space
description: >
  Mongolian Vowel Separator is not recognized as white space (eval code).
info: |
  11.2 White Space

  WhiteSpace ::
    <TAB>
    <VT>
    <FF>
    <SP>
    <NBSP>
    <ZWNBSP>
    <USP>
  <USP> ::
    Other category “Zs” code points

  General Category of U+180E is “Cf” (Format).
features: [u180e]
---*/

// U+180E between "var" and "foo".
assert.throws(SyntaxError, function() {
  eval("var\u180Efoo;");
});
