// This file was procedurally generated from the following sources:
// - src/assignment-target-type/asyncarrowfunction-5.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    AsyncArrowFunction
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

(async () => true) = 1;
