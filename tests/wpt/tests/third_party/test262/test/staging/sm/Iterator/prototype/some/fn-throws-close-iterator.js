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
    return { done: this.closed };
  }

  closed = false;
  return() {
    this.closed = true;
  }
}

const fn = () => { throw new Error(); };
const iter = new TestIterator();

assert.sameValue(iter.closed, false);
assert.throws(Error, () => iter.some(fn));
assert.sameValue(iter.closed, true);

