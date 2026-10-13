// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Non-integral Number skippedElements values throw TypeError and close the iterator
features: [iterator-includes]
---*/

let closed = false;
let iterator = {
  __proto__: Iterator.prototype,
  get next() {
    throw new Test262Error('next should not be read');
  },
  return() {
    closed = true;
    return {};
  },
};

assert.throws(TypeError, function() {
  iterator.includes(0, -0.1);
});
assert.sameValue(closed, true, 'closed on -0.1');

closed = false;
assert.throws(TypeError, function() {
  iterator.includes(0, 0.1);
});
assert.sameValue(closed, true, 'closed on 0.1');
