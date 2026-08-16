// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  skippedElements of Number.MAX_SAFE_INTEGER is accepted
features: [iterator-includes]
---*/

let returnCalls = 0;
let nextCalls = 0;
let iter = {
  __proto__: Iterator.prototype,
  next() {
    ++nextCalls;
    if (nextCalls === 1) {
      return { done: false, value: 0 };
    }
    return { done: true, value: undefined };
  },
  return() {
    ++returnCalls;
    return {};
  },
};

assert.sameValue(iter.includes(0, Number.MAX_SAFE_INTEGER), false);
assert.sameValue(returnCalls, 0);
assert.sameValue(nextCalls, 2);
