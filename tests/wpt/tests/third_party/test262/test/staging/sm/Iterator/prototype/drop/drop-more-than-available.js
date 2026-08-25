// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  %Iterator.prototype%.drop returns if the iterator is done.
info: |
  Iterator Helpers proposal 2.1.5.5 1. Repeat, while remaining > 0,
    ...
    b. Let next be ? IteratorStep(iterated).
    c. If next is false, return undefined.
features:
  - iterator-helpers
---*/

//
//
let iter = [1, 2].values().drop(3);
let result = iter.next();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);

class TestIterator extends Iterator {
  counter = 0;
  next() {
    return {done: ++this.counter >= 2, value: undefined};
  }
}

iter = new TestIterator();
let dropped = iter.drop(10);
result = dropped.next();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
assert.sameValue(iter.counter, 2);

