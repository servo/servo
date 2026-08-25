// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-static-semantics-early-errors
info: |
    It is an early Syntax Error if LeftHandSideExpression is neither an
    ObjectLiteral nor an ArrayLiteral and AssignmentTargetType of
    LeftHandSideExpression is invalid or strict.
description: Assignment with non-simple target
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

1 = 1;
