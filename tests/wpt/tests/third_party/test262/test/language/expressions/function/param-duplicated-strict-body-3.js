// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 13.1;
    It is a SyntaxError if any Identifier value occurs more than once within a FormalParameterList of a strict mode
    FunctionDeclaration or FunctionExpression.
es5id: 13.1-34-s
description: >
    Strict Mode - SyntaxError is thrown if a function declaration has
    three identical parameters with a strict mode body
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

(function (param, param, param) { 'use strict'; });
