// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-40-s
description: >
    StrictMode - SyntaxError is thrown if 'arguments' occurs as the
    Identifier of a FunctionDeclaration whose FunctionBody is
    contained in strict code
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

function arguments() { 'use strict'; }
