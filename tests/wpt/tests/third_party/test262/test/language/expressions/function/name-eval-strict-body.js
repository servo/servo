// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-38-s
description: >
    StrictMode - SyntaxError is thrown if 'eval' occurs as the
    Identifier of a FunctionExpression whose FunctionBody is contained
    in strict code
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

(function eval() {'use strict';});
