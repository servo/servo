// This file was procedurally generated from the following sources:
// - src/assignment-target-type/lefthandsideexpression-logical-and-assignment-assignmentexpression-1.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    LeftHandSideExpression &&= AssignmentExpression
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

(x &&= 1) = 1;
