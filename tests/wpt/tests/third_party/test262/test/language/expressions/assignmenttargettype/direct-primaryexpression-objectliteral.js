// This file was procedurally generated from the following sources:
// - src/assignment-target-type/primaryexpression-objectliteral.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: PrimaryExpression ObjectLiteral; Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment
---*/

$DONOTEVALUATE();

{} = 1;
