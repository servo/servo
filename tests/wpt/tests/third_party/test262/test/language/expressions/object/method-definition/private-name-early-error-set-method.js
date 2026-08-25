// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-method-definitions-static-semantics-early-errors
description: >
  Throws an early SyntaxError if a method definition has a private name.
  (getter method)
info: |
  Static Semantics: Early Errors

  PropertyDefinition : MethodDefinition
    It is a Syntax Error if PrivateBoundNames of MethodDefinition is non-empty.
negative:
  phase: parse
  type: SyntaxError
features: [class-methods-private]
---*/

$DONOTEVALUATE();

var o = {
  set #m(x) {}
};
