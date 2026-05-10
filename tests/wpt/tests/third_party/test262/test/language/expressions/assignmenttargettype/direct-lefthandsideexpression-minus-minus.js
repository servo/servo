// This file was procedurally generated from the following sources:
// - src/assignment-target-type/lefthandsideexpression-minus-minus.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: It is an early Syntax Error if AssignmentTargetType of LeftHandSideExpression is not simple. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    UpdateExpression: LeftHandSideExpression--
    It is an early Syntax Error if AssignmentTargetType of LeftHandSideExpression is not simple.

---*/

$DONOTEVALUATE();

x-- = 1;
