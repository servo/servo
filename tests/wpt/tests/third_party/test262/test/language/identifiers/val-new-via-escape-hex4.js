// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.6-10
description: >
    7.6 - SyntaxError expected: reserved words used as Identifier
    Names in UTF8: new (new)
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var n\u0065w = 123;
