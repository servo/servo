// Copyright 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-tonumber-applied-to-the-string-type
description: >
  Ensure U+180E is not recognized as whitespace, test ToNumber with dynamic string
info: |
  7.1.3.1 ToNumber Applied to the String Type

  If the grammar cannot interpret the String as an expansion of
  StringNumericLiteral, then the result of ToNumber is NaN.

  StringNumericLiteral :::
    StrWhiteSpace_opt StrNumericLiteral StrWhiteSpace_opt
  StrWhiteSpace :::
    StrWhiteSpaceChar StrWhiteSpace_opt
  StrWhiteSpaceChar :::
    WhiteSpace
    LineTerminator
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
features: [u180e]
---*/

function dynaString(s1, s2) {
  return String(s1) + String(s2);
}

// CHECK#1
assert.sameValue(Number(dynaString("", "\u180E")), NaN, 'Number(dynaString("", "\u180E")) === NaN');

// CHECK#2
assert.sameValue(+(dynaString("12345", "67890\u180E")), NaN, '+(dynaString("12345", "67890\u180E")) === NaN');

// CHECK#3
assert.sameValue(Number(dynaString("Infi", "nity\u180E")), NaN, 'Number(dynaString("Infi", "nity\u180E")) === NaN');

// CHECK#4
assert.sameValue(Number(dynaString("-Infi", "nity\u180E")), NaN, 'Number(dynaString("-Infi", "nity\u180E")) === NaN');
