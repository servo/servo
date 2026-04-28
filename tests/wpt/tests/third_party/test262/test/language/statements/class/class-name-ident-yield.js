// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-class-definitions
description: >
  `yield` is not a valid class-name identifier.
info: |
  12.1.1 Static Semantics: Early Errors

  IdentifierReference : yield

  It is a Syntax Error if the code matched by this production is contained in strict mode code.

  10.2.1 Strict Mode Code

  All parts of a ClassDeclaration or a ClassExpression are strict mode code.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

class yield {}
