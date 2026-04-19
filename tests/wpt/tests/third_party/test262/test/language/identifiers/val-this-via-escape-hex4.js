// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.6-23
description: >
    7.6 - SyntaxError expected: reserved words used as Identifier
    Names in UTF8: this (this)
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var thi\u0073 = 123;
