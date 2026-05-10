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

const fn = () => {};
const iter = new TestIterator();

assert.sameValue(iter.closed, false);
assert.throws(Error, () => iter.forEach(fn));
assert.sameValue(iter.closed, false);

