// This file was procedurally generated from the following sources:
// - src/assignment-target-type/logicalorexpression-logical-or-logicalandexpression-1.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    LogicalORExpression: LogicalORExpression || LogicalANDExpression
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

1 || 2 = 1;
