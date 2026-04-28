// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  %Iterator.prototype%.take returns if the iterator is done.
info: |
  Iterator Helpers proposal 2.1.5.4 2. Repeat,
    ...
    c. Let next be ? IteratorStep(iterated, lastValue).
    d. If next is false, return undefined.
features:
  - iterator-helpers
---*/

//
//
let iter = [1, 2].values().take(3);
for (const expected of [1, 2]) {
  const result = iter.next();
  assert.sameValue(result.value, expected);
  assert.sameValue(result.done, false);
}
let result = iter.next();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);

class TestIterator extends Iterator {
  counter = 0;
  next() {
    return {done: ++this.counter >= 2, value: undefined};
  }

  closed = false;
  return(value) {
    this.closed = true;
    return {done: true, value};
  }
}

iter = new TestIterator();
let taken = iter.take(10);
for (const value of taken) {
  assert.sameValue(value, undefined);
}
result = taken.next();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
assert.sameValue(iter.counter, 2);
assert.sameValue(iter.closed, false);

