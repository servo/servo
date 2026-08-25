// This file was procedurally generated from the following sources:
// - src/assignment-target-type/coalesceexpressionhead-coalesce-bitwiseorexpression-0.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    CoalesceExpression: CoalesceExpressionHead ?? BitwiseORExpression
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

x ?? y = 1;
