// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-4gs
description: >
    Strict Mode - SyntaxError is thrown if the identifier 'arguments'
    appears within a FormalParameterList of a strict mode
    FunctionExpression
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

var _13_1_4_fun = function (arguments) { };
