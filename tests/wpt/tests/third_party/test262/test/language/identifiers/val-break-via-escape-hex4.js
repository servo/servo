// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.6-4
description: >
    7.6 - SyntaxError expected: reserved words used as Identifier
    Names in UTF8: break (break)
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var \u0062\u0072\u0065\u0061\u006b = 123;;
