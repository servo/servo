// Copyright 2018 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Maya Lekova <mslekova@chromium.org>
esid: await
description: >
  Await on async functions and builtin Promises are properly interleaved,
  meaning await takes only 1 tick on the microtask queue.
flags: [async]
features: [async-functions]
includes: [compareArray.js]
---*/

const actual = [];
const expected = [
  'Await: 1',
  'Promise: 1',
  'Await: 2',
  'Promise: 2'
];

async function pushAwait(value) {
  actual.push('Await: ' + value);
}

async function callAsync() {
  await pushAwait(1);
  await pushAwait(2);
}

function checkAssertions() {
  assert.compareArray(actual, expected,
    'Async/await and promises should be interleaved');
}

callAsync();

new Promise(function (resolve) {
  actual.push('Promise: 1');
  resolve();
}).then(function () {
  actual.push('Promise: 2');
}).then(checkAssertions).then($DONE, $DONE);
