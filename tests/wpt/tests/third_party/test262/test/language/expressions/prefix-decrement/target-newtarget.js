// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-update-expressions-static-semantics-early-errors
description: Applied to new.target
info: |
  UnaryExpression :
    ++ UnaryExpression
    -- UnaryExpression

  - It is an early Syntax Error if IsValidSimpleAssignmentTarget of
    UnaryExpression is invalid or strict.

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
  --new.target;
}
