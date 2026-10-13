// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Underlying iterator is closed when skippedElements validation fails
features: [iterator-includes]
---*/

let closed = false;
let closable = {
  __proto__: Iterator.prototype,
  get next() {
    throw new Test262Error('next should not be read');
  },
  return() {
    closed = true;
    return {};
  },
};

assert.throws(RangeError, function() {
  closable.includes(null, -2);
});
assert.sameValue(closed, true, 'iterator closed for negative skippedElements');

closed = false;
assert.throws(RangeError, function() {
  closable.includes(null, Number.MAX_SAFE_INTEGER + 1);
});
assert.sameValue(closed, true, 'iterator closed for too-large skippedElements');

closed = false;
assert.throws(TypeError, function() {
  closable.includes(null, 'a string');
});
assert.sameValue(closed, true, 'iterator closed when skippedElements validation produces a TypeError');
