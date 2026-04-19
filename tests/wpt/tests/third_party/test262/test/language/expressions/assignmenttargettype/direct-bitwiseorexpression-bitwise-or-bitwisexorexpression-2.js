// This file was procedurally generated from the following sources:
// - src/assignment-target-type/bitwiseorexpression-bitwise-or-bitwisexorexpression-2.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    BitwiseORExpression: BitwiseORExpression | BitwiseXORExpression
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

true | false = 1;
