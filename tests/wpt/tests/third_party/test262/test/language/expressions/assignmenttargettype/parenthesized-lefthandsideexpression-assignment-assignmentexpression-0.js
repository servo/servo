// This file was procedurally generated from the following sources:
// - src/assignment-target-type/lefthandsideexpression-assignment-assignmentexpression-0.case
// - src/assignment-target-type/invalid/parenthesized.template
/*---
description: If LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral, the following Early Error rule is applied, It is a Syntax Error if AssignmentTargetType of LeftHandSideExpression is not simple. (ParenthesizedExpression)
esid: sec-grouping-operator-static-semantics-assignmenttargettype
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ParenthesizedExpression: (Expression)

    Return AssignmentTargetType of Expression.

    LeftHandSideExpression = AssignmentExpression
    If LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral, the following Early Error rule is applied, It is a Syntax Error if AssignmentTargetType of LeftHandSideExpression is not simple.

---*/

$DONOTEVALUATE();

((x = y)) = 1;
