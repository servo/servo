// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    White space cannot be expressed as a Unicode escape sequence consisting
    of six characters, namely \u plus four hexadecimal digits
es5id: 7.2_A5_T2
description: Use VERTICAL TAB (U+000B)
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var\u000Bx;
