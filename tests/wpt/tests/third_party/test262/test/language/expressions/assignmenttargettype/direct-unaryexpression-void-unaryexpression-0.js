// This file was procedurally generated from the following sources:
// - src/assignment-target-type/unaryexpression-void-unaryexpression-0.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    UnaryExpression: void UnaryExpression
    Static Semantics AssignmentTargetType, Return invalid

---*/

$DONOTEVALUATE();

void x = 1;
