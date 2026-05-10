// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
description: >
  `yield` is a reserved identifier in strict mode code and may not be used as a label.
info: |
  Identifier : IdentifierName but not ReservedWord

  It is a Syntax Error if this phrase is contained in strict mode code and the
  StringValue of IdentifierName is: "implements", "interface", "let", "package",
  "private", "protected", "public", "static", or "yield".
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

yi\u0065ld: 1;
