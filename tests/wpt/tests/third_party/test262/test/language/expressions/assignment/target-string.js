// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-static-semantics-early-errors
description: >
    simple assignment throws SyntaxError if LeftHandSide is not a
    reference (string)
info: |
    AssignmentExpression : LeftHandSideExpression = AssignmentExpression

    It is an early Syntax Error if LeftHandSideExpression is neither an
    ObjectLiteral nor an ArrayLiteral and AssignmentTargetType of
    LeftHandSideExpression is invalid or strict.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

'x' = 42;
