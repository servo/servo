// This file was procedurally generated from the following sources:
// - src/assignment-target-type/unaryexpression-minus-unaryexpression-1.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    UnaryExpression: - UnaryExpression
    Static Semantics AssignmentTargetType, Return invalid

---*/

$DONOTEVALUATE();

-1 = 1;
