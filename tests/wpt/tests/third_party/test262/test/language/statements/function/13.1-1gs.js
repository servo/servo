// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-1gs
description: >
    Strict Mode - SyntaxError is thrown if the identifier 'eval'
    appears within a FormalParameterList of a strict mode
    FunctionDeclaration
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

function _13_1_1_fun(eval) { }
