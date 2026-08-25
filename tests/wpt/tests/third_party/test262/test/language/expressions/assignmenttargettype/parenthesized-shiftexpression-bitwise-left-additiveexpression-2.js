// This file was procedurally generated from the following sources:
// - src/assignment-target-type/shiftexpression-bitwise-left-additiveexpression-2.case
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

    ShiftExpression << AdditiveExpression
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

(true << false) = 1;
