// Copyright (C) 2021 Stuart Cook. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-for-in-and-for-of-statements
description: >
  The left-hand-side of a for-of loop may be the identifier `async`
  surrounded by parentheses.
info: |
  ForInOfStatement[Yield, Await, Return] :
    for ( [lookahead ∉ { let, async of }] LeftHandSideExpression[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
---*/

let async;

for ((async) of [7]);

assert.sameValue(async, 7);
