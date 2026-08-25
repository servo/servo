// This file was procedurally generated from the following sources:
// - src/assignment-target-type/updateexpression-star-star-exponentiationexpression-1.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
features: [exponentiation]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    UpdateExpression ** ExponentiationExpression
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

1 ** 2 = 1;
