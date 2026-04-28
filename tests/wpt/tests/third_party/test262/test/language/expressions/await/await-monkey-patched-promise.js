// Copyright 2018 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Maya Lekova <mslekova@chromium.org>
esid: await
description: >
  This test demonstrates that monkey-patched "then" on native promises will
  not get called. Adapted from example by Kevin Smith:
  https://github.com/tc39/ecma262/pull/1250#issuecomment-401082195
flags: [async]
features: [async-functions]
includes: [compareArray.js]
---*/

let thenCallCount = 0;
const value = 42;

const actual = [];
const expected = [
  'Promise: 1',
  'Await: ' + value,
  'Promise: 2',
];

const patched = Promise.resolve(value);
patched.then = function(...args) {
  thenCallCount++;
  Promise.prototype.then.apply(this, args);
};

async function trigger() {
  actual.push('Await: ' + await patched);
}

function checkAssertions() {
  assert.compareArray(actual, expected,
    'Async/await and promises should be interleaved');
  assert.sameValue(thenCallCount, 0,
    'Monkey-patched "then" on native promises should not be called.')
}

trigger().then(checkAssertions).then($DONE, $DONE);

new Promise(function (resolve) {
  actual.push('Promise: 1');
  resolve();
}).then(function () {
  actual.push('Promise: 2');
});
