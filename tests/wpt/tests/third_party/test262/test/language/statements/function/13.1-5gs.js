// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-5gs
description: >
    Strict Mode - SyntaxError is thrown if a FunctionDeclaration has
    two identical parameters
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

function _13_1_5_fun(param, param) { }
