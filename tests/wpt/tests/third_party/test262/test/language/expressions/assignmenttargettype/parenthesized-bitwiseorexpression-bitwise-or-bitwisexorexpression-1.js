// This file was procedurally generated from the following sources:
// - src/assignment-target-type/bitwiseorexpression-bitwise-or-bitwisexorexpression-1.case
// - src/assignment-target-type/invalid/parenthesized.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (ParenthesizedExpression)
esid: sec-grouping-operator-static-semantics-assignmenttargettype
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ParenthesizedExpression: (Expression)

    Return AssignmentTargetType of Expression.

    BitwiseORExpression: BitwiseORExpression | BitwiseXORExpression
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

(1 | 2) = 1;
