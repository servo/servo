// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function-definitions-runtime-semantics-instantiatefunctionobject
description: Function.prototype.toString line terminator normalisation (CR-LF)
info: |
  Function.prototype.toString should not normalise line terminator sequences to Line Feed characters.
  This file uses (Carriage Return, Line Feed) sequences as line terminators.
includes: [nativeFunctionMatcher.js]
---*/

// before
function
// a
f
// b
(
// c
x
// d
,
// e
y
// f
)
// g
{
// h
;
// i
;
// j
}
// after

assertToStringOrNativeFunction(f, "function\r\n// a\r\nf\r\n// b\r\n(\r\n// c\r\nx\r\n// d\r\n,\r\n// e\r\ny\r\n// f\r\n)\r\n// g\r\n{\r\n// h\r\n;\r\n// i\r\n;\r\n// j\r\n}");
