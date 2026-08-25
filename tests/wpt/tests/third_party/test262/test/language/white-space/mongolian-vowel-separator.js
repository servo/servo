// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-white-space
description: >
  Mongolian Vowel Separator is not recognized as white space.
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
negative:
  phase: parse
  type: SyntaxError
features: [u180e]
---*/

$DONOTEVALUATE();

// U+180E between "var" and "foo"; UTF8(0x180E) = 0xE1 0xA0 0x8E
var᠎foo;
