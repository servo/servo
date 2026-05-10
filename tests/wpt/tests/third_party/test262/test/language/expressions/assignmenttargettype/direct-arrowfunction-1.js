// This file was procedurally generated from the following sources:
// - src/assignment-target-type/arrowfunction-1.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    ArrowFunction
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

() => ({}) = 1;
