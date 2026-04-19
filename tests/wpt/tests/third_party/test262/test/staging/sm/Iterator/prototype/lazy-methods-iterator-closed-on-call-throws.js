// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  Lazy %Iterator.prototype% methods close the iterator if callback throws.
info: |
  Iterator Helpers proposal 2.1.5
features:
  - iterator-helpers
---*/

//
//
class TestError extends Error {}
class TestIterator extends Iterator {
  next() {
    return {done: false, value: 1};
  }

  closed = false;
  return() {
    this.closed = true;
    return {done: true};
  }
}

function fn() {
  throw new TestError();
}
const methods = [
  iter => iter.map(fn),
  iter => iter.filter(fn),
  iter => iter.flatMap(fn),
];

for (const method of methods) {
  const iter = new TestIterator();
  assert.sameValue(iter.closed, false);
  assert.throws(TestError, () => method(iter).next());
  assert.sameValue(iter.closed, true);
}

