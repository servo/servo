// This file was procedurally generated from the following sources:
// - src/assignment-target-type/callexpression-as-for-in-lhs.case
// - src/assignment-target-type/invalid/statement/default.template
/*---
description: Static Semantics AssignmentTargetType, Return web-compat. (Direct assignment)
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    CallExpression :
      CoverCallExpressionAndAsyncArrowHead
      CallExpression Arguments
    1. If the host is a web browser or otherwise supports Runtime Errors for Function Call Assignment Targets, then
       a. If IsStrict(this CallExpression) is false, return ~web-compat~.
    2. Return ~invalid~.

---*/

$DONOTEVALUATE();

for (f() in [1]) {}
