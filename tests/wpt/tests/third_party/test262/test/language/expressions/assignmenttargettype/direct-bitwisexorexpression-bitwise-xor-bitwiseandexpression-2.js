// This file was procedurally generated from the following sources:
// - src/assignment-target-type/bitwisexorexpression-bitwise-xor-bitwiseandexpression-2.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    BitwiseXORExpression: BitwiseXORExpression ^ BitwiseANDExpression
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

true ^ false = 1;
