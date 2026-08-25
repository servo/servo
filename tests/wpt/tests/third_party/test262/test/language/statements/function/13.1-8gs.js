// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-8gs
description: >
    Strict Mode - SyntaxError is thrown if a FunctionExpression has
    two identical parameters
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

var _13_1_8_fun = function (param, param) { };
