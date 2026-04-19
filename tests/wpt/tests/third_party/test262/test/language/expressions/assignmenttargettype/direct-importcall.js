// This file was procedurally generated from the following sources:
// - src/assignment-target-type/importcall.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
flags: [generated, module]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment

    ImportCall
    Static Semantics AssignmentTargetType, Return invalid.

---*/

$DONOTEVALUATE();

import() = 1;
