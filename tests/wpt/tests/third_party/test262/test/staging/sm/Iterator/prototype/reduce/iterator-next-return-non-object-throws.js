// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - iterator-helpers
info: |
  Iterator is not enabled unconditionally
description: |
  pending
esid: pending
---*/

class TestIterator extends Iterator {
  constructor(value) {
    super();
    this.value = value;
  }

  next() {
    return this.value;
  }
}

const sum = (x, y) => x + y;

let iter = new TestIterator(undefined);
assert.throws(TypeError, () => iter.reduce(sum));
iter = new TestIterator(null);
assert.throws(TypeError, () => iter.reduce(sum));
iter = new TestIterator(0);
assert.throws(TypeError, () => iter.reduce(sum));
iter = new TestIterator(false);
assert.throws(TypeError, () => iter.reduce(sum));
iter = new TestIterator('');
assert.throws(TypeError, () => iter.reduce(sum));
iter = new TestIterator(Symbol(''));
assert.throws(TypeError, () => iter.reduce(sum));

