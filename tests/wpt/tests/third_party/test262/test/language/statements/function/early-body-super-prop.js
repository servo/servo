// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function-definitions-static-semantics-early-errors
es6id: 14.1.2
description: Body may not contain a "super" property reference
info: |
  It is a Syntax Error if FunctionBody Contains SuperProperty is true.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function f() {
  super.x;
}
