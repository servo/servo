// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.6-24
description: >
    7.6 - SyntaxError expected: reserved words used as Identifier
    Names in UTF8: if (if)
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var \u0069\u0066 = 123;
