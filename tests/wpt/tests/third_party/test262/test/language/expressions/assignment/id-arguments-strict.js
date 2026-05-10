// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es5id: 11.13.1-4-30-s
description: >
  Strict Mode - SyntaxError is thrown if the identifier 'arguments' appears as
  the LeftHandSideExpression (PrimaryExpression) of simple assignment(=).
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

(arguments) = 20;
