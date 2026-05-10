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
class Iter {
  next(value) {
    assert.sameValue(arguments.length, 0);
    return { done: false, value };
  }
}

const iter = new Iter();
const wrap = Iterator.from(iter);
assert.sameValue(iter !== wrap, true);

assert.sameValue(iter.v, undefined);
wrap.next(1);

