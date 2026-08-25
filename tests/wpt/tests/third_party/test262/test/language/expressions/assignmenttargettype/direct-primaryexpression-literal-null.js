// This file was procedurally generated from the following sources:
// - src/assignment-target-type/primaryexpression-literal-null.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: PrimaryExpression Literal NullLiteral; Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment
---*/

$DONOTEVALUATE();

null = 1;
