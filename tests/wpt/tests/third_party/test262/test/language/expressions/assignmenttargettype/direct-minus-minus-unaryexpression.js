// This file was procedurally generated from the following sources:
// - src/assignment-target-type/minus-minus-unaryexpression.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: It is an early Syntax Error if AssignmentTargetType of UnaryExpression is not simple. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    UpdateExpression: --UnaryExpression
    It is an early Syntax Error if AssignmentTargetType of UnaryExpression is not simple.

---*/

$DONOTEVALUATE();

--x = 1;
