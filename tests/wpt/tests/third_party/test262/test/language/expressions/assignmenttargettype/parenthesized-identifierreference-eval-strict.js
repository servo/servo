// This file was procedurally generated from the following sources:
// - src/assignment-target-type/identifierreference-eval-strict.case
// - src/assignment-target-type/invalid/parenthesized.template
/*---
description: If this IdentifierReference is contained in strict mode code and StringValue of Identifier is "eval" or "arguments", return invalid. (ParenthesizedExpression)
esid: sec-grouping-operator-static-semantics-assignmenttargettype
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    ParenthesizedExpression: (Expression)

    Return AssignmentTargetType of Expression.
---*/

$DONOTEVALUATE();

(eval) = 1;
