// Copyright 2018 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Maya Lekova <mslekova@chromium.org>
esid: await
description: >
  This test demonstrates that "then" on a non-native promise
  will still get called.
flags: [async]
features: [async-functions]
includes: [compareArray.js]
---*/

let thenCallCount = 0;

const actual = [];
const expected = [
  'Promise: 1',
  'Promise: 2',
  'Await: 1',
  'Promise: 3',
  'Promise: 4',
  'Await: 2',
];

const patched = {};
patched.then = function(fulfill, reject) {
  thenCallCount++;
  fulfill(thenCallCount);
};

async function trigger() {
  actual.push('Await: ' + await patched);
  actual.push('Await: ' + await patched);
}

function checkAssertions() {
  assert.compareArray(actual, expected,
    'Async/await and promises should be interleaved');
  assert.sameValue(thenCallCount, 2,
    '"then" on non-native promises should be called.');
}

trigger().then(checkAssertions).then($DONE, $DONE);

new Promise(function (resolve) {
  actual.push('Promise: 1');
  resolve();
}).then(function () {
  actual.push('Promise: 2');
}).then(function () {
  actual.push('Promise: 3');
}).then(function () {
  actual.push('Promise: 4');
});
