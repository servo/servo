// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.6-29
description: >
    7.6 - SyntaxError expected: reserved words used as Identifier
    Names in UTF8: delete (delete)
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var \u0064elete = 123;
