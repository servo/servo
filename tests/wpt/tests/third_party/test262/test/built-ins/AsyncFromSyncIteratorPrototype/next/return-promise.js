// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.next
description: >
  "next" returns a promise for an IteratorResult object
info: |
  %AsyncFromSyncIteratorPrototype%.next ( value )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  18. Return promiseCapability.[[Promise]].

features: [async-iteration]
---*/

function* g() {
}

async function* asyncg() {
  yield* g();
}

var result = asyncg().next();
assert(result instanceof Promise)
