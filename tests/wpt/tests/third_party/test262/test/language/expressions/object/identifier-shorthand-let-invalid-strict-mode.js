// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Object literal shorthands are limited to valid identifier references. Future Reserved Words are disallowed in Strict Mode. (let)
esid: sec-object-initializer
flags: [noStrict]
info: |
  PropertyDefinition:
    IdentifierReference
    CoverInitializedName
    PropertyName : AssignmentExpression
    MethodDefinition

  Identifier : IdentifierName but not ReservedWord
    It is a Syntax Error if this phrase is contained in strict mode code and
    the StringValue of IdentifierName is: "implements", "interface", "let",
    "package", "private", "protected", "public", "static", or "yield". 
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var let = 1;
(function() {
  "use strict";
  ({
    let
  });
});
