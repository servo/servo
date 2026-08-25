// This file was procedurally generated from the following sources:
// - src/assignment-target-type/primaryexpression-functionexpression.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: PrimaryExpression FunctionExpression, Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment
---*/

$DONOTEVALUATE();

function() {} = 1;
