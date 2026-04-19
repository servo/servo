// Copyright (C) 2021 Stuart Cook. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-for-in-and-for-of-statements
description: >
  The left-hand-side of a for-await-of loop may be the identifier `async`.
info: |
  ForInOfStatement[Yield, Await, Return]:
    [+Await] for await ( [lookahead ≠ let] LeftHandSideExpression[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
features: [async-iteration]
flags: [async]
---*/

let async;

async function fn() {
  for await (async of [7]);
}

fn()
  .then(() => assert.sameValue(async, 7), $DONE)
  .then($DONE, $DONE);
