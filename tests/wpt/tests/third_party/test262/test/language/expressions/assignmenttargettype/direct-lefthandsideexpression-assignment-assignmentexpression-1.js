// This file was procedurally generated from the following sources:
// - src/assignment-target-type/lefthandsideexpression-assignment-assignmentexpression-1.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: If LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral, the following Early Error rule is applied, It is a Syntax Error if AssignmentTargetType of LeftHandSideExpression is not simple. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    LeftHandSideExpression = AssignmentExpression
    If LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral, the following Early Error rule is applied, It is a Syntax Error if AssignmentTargetType of LeftHandSideExpression is not simple.

---*/

$DONOTEVALUATE();

(x = 1) = 1;
