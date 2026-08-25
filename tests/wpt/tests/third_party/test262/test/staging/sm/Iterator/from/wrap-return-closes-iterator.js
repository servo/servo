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
  next() {
    if (this.closed)
      return { done: true, value: undefined };
    return { done: false, value: 0 };
  }

  return(value) {
    assert.sameValue(arguments.length, 0);
    this.closed = true;
    return { done: true, value: 42 };
  }
}

const iter = new Iter();
const wrap = Iterator.from(iter);
assert.sameValue(iter.closed, undefined);

let result = wrap.next();
assert.sameValue(result.done, false);
assert.sameValue(result.value, 0);

result = wrap.return(1);
assert.sameValue(result.done, true);
assert.sameValue(result.value, 42);

assert.sameValue(iter.closed, true);
result = wrap.next();
assert.sameValue(result.done, true);
assert.sameValue(result.value, undefined);

