// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.13.2-6-9-s
esid: sec-assignment-operators
description: >
    Strict Mode - SyntaxError is thrown if the identifier eval appear
    as the LeftHandSideExpression of a Compound Assignment operator(&=)
flags: [onlyStrict]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

eval &= 20;
