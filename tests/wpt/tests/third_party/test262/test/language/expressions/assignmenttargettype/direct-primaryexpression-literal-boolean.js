// This file was procedurally generated from the following sources:
// - src/assignment-target-type/primaryexpression-literal-boolean.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: PrimaryExpression Literal BooleanLiteral; Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment
---*/

$DONOTEVALUATE();

true = 1;
