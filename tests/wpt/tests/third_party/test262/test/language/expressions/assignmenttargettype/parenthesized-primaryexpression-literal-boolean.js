// This file was procedurally generated from the following sources:
// - src/assignment-target-type/primaryexpression-literal-boolean.case
// - src/assignment-target-type/invalid/parenthesized.template
/*---
description: PrimaryExpression Literal BooleanLiteral; Return invalid. (ParenthesizedExpression)
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

(true) = 1;
