// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
es5id: 7.6.1-25-s
description: >
    7.6 - SyntaxError expected: reserved words used as Identifier
    Names in UTF8: \u0073\u0074\u0061\u0074\u0069\u0063 (static)
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

var \u0073\u0074\u0061\u0074\u0069\u0063 = 123;
