// This file was procedurally generated from the following sources:
// - src/assignment-target-type/primaryexpression-asyncfunctionexpression.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: PrimaryExpression AsyncFunctionExpression; Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment
---*/

$DONOTEVALUATE();

async function () {} = 1;
