// Copyright (C) 2017 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Object literal shorthands are only valid with identifier references,
  not computed property names.
esid: sec-object-initializer
info: |
  PropertyDefinition:
    IdentifierReference
    CoverInitializedName
    PropertyName : AssignmentExpression
    MethodDefinition

  PropertyName:
    LiteralPropertyName
    ComputedPropertyName
negative:
  phase: parse
  type: SyntaxError
---*/

var x = "y";
var y = 42;

$DONOTEVALUATE();

({[x]});
