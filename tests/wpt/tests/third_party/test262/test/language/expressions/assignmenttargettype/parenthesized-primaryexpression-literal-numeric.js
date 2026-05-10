// This file was procedurally generated from the following sources:
// - src/assignment-target-type/primaryexpression-literal-numeric.case
// - src/assignment-target-type/invalid/parenthesized.template
/*---
description: PrimaryExpression Literal NumericLiteral; Return invalid. (ParenthesizedExpression)
esid: sec-grouping-operator-static-semantics-assignmenttargettype
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ParenthesizedExpression: (Expression)

    Return AssignmentTargetType of Expression.
---*/

$DONOTEVALUATE();

(0) = 1;
