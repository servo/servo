// This file was procedurally generated from the following sources:
// - src/assignment-target-type/importcall-source.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: Static Semantics AssignmentTargetType, Return invalid. (Direct assignment)
features: [source-phase-imports]
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

import.source() = 1;
