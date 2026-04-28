// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 13.1;
    It is a SyntaxError if any Identifier value occurs more than once within a FormalParameterList of a strict mode
    FunctionDeclaration or FunctionExpression.
es5id: 13.1-28-s
description: >
    Strict Mode - SyntaxError is thrown if a function is created using
    a FunctionDeclaration whose FunctionBody is contained in strict
    code and the function has three identical parameters
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

function _13_1_28_fun(param, param, param) { 'use strict'; }
