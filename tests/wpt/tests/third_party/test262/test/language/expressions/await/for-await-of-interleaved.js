// Copyright 2018 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Maya Lekova <mslekova@chromium.org>
esid: await
description: >
  for-await-of iteration and builtin Promises are properly interleaved,
  meaning await in for-of loop takes only 1 tick on the microtask queue.
flags: [async]
features: [async-functions, async-iteration, generators]
includes: [compareArray.js]
---*/

const actual = [];
const expected = [
  'Promise: 6',
  'Promise: 5',
  'Await: 3',
  'Promise: 4',
  'Promise: 3',
  'Await: 2',
  'Promise: 2',
  'Promise: 1',
  'Await: 1',
  'Promise: 0'
];
const iterations = 3;

async function* naturalNumbers(start) {
  let current = start;
  while (current > 0) {
    yield Promise.resolve(current--);
  }
}

async function trigger() {
  for await (const num of naturalNumbers(iterations)) {
    actual.push('Await: ' + num);
  }
}

async function checkAssertions() {
  assert.compareArray(actual, expected,
    'Async/await and promises should be interleaved');
}

function countdown(counter) {
  actual.push('Promise: ' + counter);
  if (counter > 0) {
    return Promise.resolve(counter - 1).then(countdown);
  } else {
    checkAssertions().then($DONE, $DONE);
  }
}

trigger();
countdown(iterations * 2);
