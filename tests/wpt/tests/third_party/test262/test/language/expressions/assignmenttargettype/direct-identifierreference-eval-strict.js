// This file was procedurally generated from the following sources:
// - src/assignment-target-type/identifierreference-eval-strict.case
// - src/assignment-target-type/invalid/direct.template
/*---
description: If this IdentifierReference is contained in strict mode code and StringValue of Identifier is "eval" or "arguments", return invalid. (Direct assignment)
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    Direct assignment
---*/

$DONOTEVALUATE();

eval = 1;
