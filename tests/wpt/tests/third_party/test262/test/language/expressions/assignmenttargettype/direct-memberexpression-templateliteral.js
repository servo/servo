// This file was procedurally generated from the following sources:
// - src/assignment-target-type/memberexpression-templateliteral.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    MemberExpression TemplateLiteral
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

o.f()`` = 1;
