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
  next() {
    throw new Error();
  }

  closed = false;
  return() {
    this.closed = true;
  }
}

const sum = (x, y) => x + y;
const iter = new TestIterator();

assert.sameValue(iter.closed, false);
assert.throws(Error, () => iter.reduce(sum));
assert.sameValue(iter.closed, false);

