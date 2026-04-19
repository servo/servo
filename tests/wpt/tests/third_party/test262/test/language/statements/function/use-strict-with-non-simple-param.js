// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function-definitions-static-semantics-early-errors
description: >
  A SyntaxError is thrown if a function contains a non-simple parameter list and a UseStrict directive.
info: |
  Static Semantics: Early Errors

  It is a Syntax Error if ContainsUseStrict of FunctionBody is true and IsSimpleParameterList of FormalParameters is false.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function f(a = 0) {
  "use strict";
}
