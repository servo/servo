// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arrow-function-definitions-static-semantics-early-errors
description: >
  A SyntaxError is thrown if an arrow function contains a non-simple parameter list and a UseStrict directive.
info: |
  Static Semantics: Early Errors

  It is a Syntax Error if ContainsUseStrict of ConciseBody is true and IsSimpleParameterList of ArrowParameters is false.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var f = (a = 0) => {
  "use strict";
};
