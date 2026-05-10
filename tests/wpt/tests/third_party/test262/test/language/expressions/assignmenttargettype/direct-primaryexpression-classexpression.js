// This file was procedurally generated from the following sources:
// - src/assignment-target-type/primaryexpression-classexpression.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: PrimaryExpression ClassExpression; Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment
---*/

$DONOTEVALUATE();

class {} = 1;
