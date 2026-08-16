// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Gets the next method from the iterator only once
features: [iterator-includes]
---*/

let nextGets = 0;

let iterator = {
  __proto__: Iterator.prototype,
  get next() {
    ++nextGets;
    let counter = 5;
    return function() {
      if (counter < 0) {
        return { done: true, value: undefined };
      }
      return { done: false, value: --counter };
    };
  }
};

assert.sameValue(nextGets, 0);
assert.sameValue(iterator.includes(3), true);
assert.sameValue(nextGets, 1);
