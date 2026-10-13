// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Iterator.prototype.windows supports a this value that does not inherit from
  Iterator.prototype but implements the iterator protocol
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

features: [iterator-chunking]
includes: [compareArray.js]
---*/
let iter = {
  get next() {
    let count = 3;
    return function () {
      --count;
      return count >= 0 ? { done: false, value: count } : { done: true, value: undefined };
    };
  }
};

let windowed = Iterator.prototype.windows.call(iter, 2);

let result = windowed.next();
assert.compareArray(result.value, [2, 1]);
assert.sameValue(result.done, false);

result = windowed.next();
assert.compareArray(result.value, [1, 0]);
assert.sameValue(result.done, false);

result = windowed.next();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
