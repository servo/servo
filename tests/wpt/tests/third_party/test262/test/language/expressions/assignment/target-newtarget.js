// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-assignment-operators-static-semantics-early-errors
description: Applied to new.target
info: |
  AssignmentExpression : LeftHandSideExpression = AssignmentExpression

  - It is an early Syntax Error if LeftHandSideExpression is neither an
    ObjectLiteral nor an ArrayLiteral and AssignmentTargetType of
    LeftHandSideExpression is invalid or strict.

  12.3.1.6 Static Semantics: AssignmentTargetType

  NewTarget:

  new.target

  1. Return invalid.
negative:
  phase: parse
  type: SyntaxError
features: [new.target]
---*/

$DONOTEVALUATE();

function f() {
  new.target = 1;
}
