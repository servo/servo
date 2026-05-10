// This file was procedurally generated from the following sources:
// - src/assignment-target-type/primaryexpression-literal-string.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: PrimaryExpression Literal StringLiteral; Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment
---*/

$DONOTEVALUATE();

'' = 1;
