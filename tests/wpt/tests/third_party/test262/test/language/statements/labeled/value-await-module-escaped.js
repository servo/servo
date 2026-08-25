// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
description: >
  `await` is a reserved identifier in module code and may not be used as a label.
info: |
  Identifier : IdentifierName but not ReservedWord

  It is a Syntax Error if the goal symbol of the syntactic grammar is Module and
  the StringValue of IdentifierName is "await".
negative:
  phase: parse
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

aw\u0061it: 1;
