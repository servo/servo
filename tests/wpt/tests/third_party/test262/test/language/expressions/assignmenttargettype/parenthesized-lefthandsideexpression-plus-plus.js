// This file was procedurally generated from the following sources:
// - src/assignment-target-type/lefthandsideexpression-plus-plus.case
// - src/assignment-target-type/invalid/parenthesized.template
/*---
description: It is an early Syntax Error if AssignmentTargetType of LeftHandSideExpression is not simple. (ParenthesizedExpression)
esid: sec-grouping-operator-static-semantics-assignmenttargettype
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ParenthesizedExpression: (Expression)

    Return AssignmentTargetType of Expression.

    UpdateExpression: LeftHandSideExpression ++
    It is an early Syntax Error if AssignmentTargetType of LeftHandSideExpression is not simple.

---*/

$DONOTEVALUATE();

(x++) = 1;
