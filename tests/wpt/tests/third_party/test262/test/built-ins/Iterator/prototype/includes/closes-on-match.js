// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Iterator.prototype.includes closes the iterator after a successful match
features: [iterator-includes]
---*/

let closed = false;
let i = 0;
let iter = {
  __proto__: Iterator.prototype,
  next() {
    ++i;
    return { done: false, value: i };
  },
  return() {
    closed = true;
    return {};
  },
};

assert.sameValue(iter.includes(5), true);
assert.sameValue(closed, true);
assert.sameValue(i, 5);
