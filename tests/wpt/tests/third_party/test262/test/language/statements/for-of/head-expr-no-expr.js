// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Expression not allowed in head's AssignmentExpression position
info: |
    IterationStatement :
        for ( LeftHandSideExpression of AssignmentExpression ) Statement
es6id: 13.7
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var x;
for (x of [], []) {}
