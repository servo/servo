// This file was procedurally generated from the following sources:
// - src/assignment-target-type/new-memberexpression-arguments.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    new MemberExpression Arguments
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

new f() = 1;
