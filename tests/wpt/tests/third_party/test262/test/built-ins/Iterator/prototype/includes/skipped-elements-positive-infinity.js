// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  skippedElements of +Infinity always returns false after natural exhaustion
features: [iterator-includes]
---*/

let returnCalls = 0;
let count = 0;

let iter = {
  __proto__: Iterator.prototype,
  next() {
    ++count;
    if (count < 4) {
      return { done: false, value: count };
    }
    return { done: true, value: undefined };
  },
  return() {
    ++returnCalls;
    return {};
  },
};

assert.sameValue(iter.includes(1, Infinity), false);
assert.sameValue(returnCalls, 0);
assert.sameValue(count, 4);
