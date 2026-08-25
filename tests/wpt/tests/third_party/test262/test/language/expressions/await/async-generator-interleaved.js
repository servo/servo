// Copyright 2018 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Maya Lekova <mslekova@chromium.org>
esid: await
description: >
  Await on async generator functions and builtin Promises are properly
  interleaved, meaning await takes only 1 tick on the microtask queue.
flags: [async]
features: [async-functions, async-iteration]
includes: [compareArray.js]
---*/

const actual = [];
const expected = [ 'await', 1, 'await', 2 ];
const iterations = 2;

async function pushAwait() {
  actual.push('await');
}

async function* callAsync() {
  for (let i = 0; i < iterations; i++) {
    await pushAwait();
  }
  return 0;
}

function checkAssertions() {
  assert.compareArray(actual, expected,
    'Async/await and promises should be interleaved');
}

callAsync().next();

new Promise(function (resolve) {
  actual.push(1);
  resolve();
}).then(function () {
  actual.push(2);
}).then(checkAssertions).then($DONE, $DONE);
