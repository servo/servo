// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - iterator-helpers
description: |
  pending
esid: pending
---*/

class TestError extends Error {}

class TestIterator extends Iterator {
  next() {
    return {done: false, value: 'value'};
  }

  closed = false;
  return(value) {
    this.closed = true;
    return {done: true, value};
  }
}

function checkIterResult({done, value}, expectedDone, expectedValue) {
  assert.sameValue(done, expectedDone);
  assert.sameValue(Array.isArray(value) ? value[1] : value, expectedValue);
}

const otherGlobal = $262.createRealm().global;

const methods = [
  ["map", x => x],
  ["filter", x => true],
  ["take", Infinity],
  ["drop", 0],
  ["flatMap", x => [x]],
];

for (const [method, arg] of methods) {
  const {next: otherNext} = Object.getPrototypeOf(
    new otherGlobal.Array().values()[method](arg)
  );
  const iterator = new TestIterator();
  const helper = iterator[method](arg);
  checkIterResult(otherNext.call(helper), false, 'value');
}

for (const [method, arg] of methods) {
  const {return: otherReturn} = Object.getPrototypeOf(
    new otherGlobal.Array().values()[method](arg)
  );
  const iterator = new TestIterator();
  const helper = iterator[method](arg);
  assert.sameValue(iterator.closed, false);
  checkIterResult(otherReturn.call(helper), true, undefined);
  assert.sameValue(iterator.closed, true);
}

