// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators
description: >
    Strict Mode - SyntaxError is thrown if the identifier arguments
    appear as the LeftHandSideExpression of a Logical Assignment
    operator(&&=)
flags: [onlyStrict]
negative:
  phase: parse
  type: SyntaxError
features: [logical-assignment-operators]
---*/
$DONOTEVALUATE();

arguments &&= 20;
