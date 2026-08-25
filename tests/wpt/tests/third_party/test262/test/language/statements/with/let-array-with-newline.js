// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-with-statement
description: >
  ExpressionStatement has a lookahead restriction for `let [`.
info: |
  ExpressionStatement[Yield, Await] :
    [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
    Expression[+In, ?Yield, ?Await] ;
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

// Wrapped in an if-statement to avoid reference errors at runtime.
if (false) {
    with ({}) let
    [a] = 0;
}
