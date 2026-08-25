// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-strict-mode-of-ecmascript
description: >
    eval allowed as formal parameter name of a non-strict function declaration
flags: [noStrict]
---*/

let exprCallCount = 0;
let evalValue = {};

function foo(eval) {
  assert.sameValue(eval, evalValue);
  exprCallCount += 1;
}

foo(evalValue);

assert.sameValue(exprCallCount, 1);
