// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
description: >
  `await` is a reserved identifier in module code and may not be used as a label.
info: |
  LabelIdentifier : await

  It is a Syntax Error if the goal symbol of the syntactic grammar is Module.
negative:
  phase: parse
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

await: 1;
