// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Underlying iterator is closed when argument validation fails
info: |
  %Iterator.prototype%.reduce ( reducer, [ initialValue ] )

features: [iterator-helpers]
flags: []
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

assert.throws(TypeError, function() {
  closable.reduce();
});
assert.sameValue(closed, true);

closed = false;
assert.throws(TypeError, function() {
  closable.reduce({});
});
assert.sameValue(closed, true);
