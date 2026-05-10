// This file was procedurally generated from the following sources:
// - src/assignment-target-type/optionalexpression.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
features: [optional-chaining]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    OptionalExpression
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

x?.y = 1;
