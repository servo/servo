// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
  Iterator.from returns O if it is iterable, an iterator, and an instance of Iterator.

  Iterator is not enabled unconditionally
features:
  - iterator-helpers
description: |
  pending
esid: pending
---*/
class TestIterator extends Iterator {
  [Symbol.iterator]() {
    return this;
  }

  next() {
    return { done: false, value: this.value++ };
  }

  value = 0;
}

const iter = new TestIterator();
assert.sameValue(iter, Iterator.from(iter));

const arrayIter = [1, 2, 3][Symbol.iterator]();
assert.sameValue(arrayIter, Iterator.from(arrayIter));

