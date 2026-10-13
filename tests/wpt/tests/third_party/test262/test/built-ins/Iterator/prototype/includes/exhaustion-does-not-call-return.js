// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Exhausting the iterator without finding a match does not call return
features: [iterator-includes]
---*/

let index = 0;
let returnCalls = 0;

let iterator = {
  __proto__: Iterator.prototype,
  next() {
    ++index;
    if (index <= 3) {
      return { done: false, value: index };
    }
    return { done: true, value: undefined };
  },
  get return() {
    throw new Test262Error('return should not be read');
  },
};

assert.sameValue(iterator.includes(99), false);
assert.sameValue(returnCalls, 0);
