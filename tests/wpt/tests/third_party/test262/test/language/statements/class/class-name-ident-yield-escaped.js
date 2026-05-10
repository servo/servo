// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-class-definitions
description: >
  `yield` with escape sequence is not a valid class-name identifier.
info: |
  12.1.1 Static Semantics: Early Errors

  Identifier : IdentifierName but not ReservedWord

  It is a Syntax Error if this phrase is contained in strict mode code and the
  StringValue of IdentifierName is: "implements", "interface", "let", "package",
  "private", "protected", "public", "static", or "yield".

  10.2.1 Strict Mode Code

  All parts of a ClassDeclaration or a ClassExpression are strict mode code.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

class yi\u0065ld {}
