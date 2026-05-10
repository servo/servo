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
    return { done: false, value: 0 };
  }
}

const iter = new TestIterator();
assert.throws(TypeError, () => iter.reduce());
assert.throws(TypeError, () => iter.reduce(undefined));
assert.throws(TypeError, () => iter.reduce(null));
assert.throws(TypeError, () => iter.reduce(0));
assert.throws(TypeError, () => iter.reduce(false));
assert.throws(TypeError, () => iter.reduce(''));
assert.throws(TypeError, () => iter.reduce(Symbol('')));
assert.throws(TypeError, () => iter.reduce({}));

