// This file was procedurally generated from the following sources:
// - src/assignment-target-type/primaryexpression-generatorexpression.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: PrimaryExpression ArrayLiteral; Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment
---*/

$DONOTEVALUATE();

function * () {} = 1;
