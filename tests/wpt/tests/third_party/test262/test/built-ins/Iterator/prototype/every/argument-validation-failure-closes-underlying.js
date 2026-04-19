// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.every
description: >
  Underlying iterator is closed when argument validation fails
info: |
  %Iterator.prototype%.every ( predicate )

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
  closable.every();
});
assert.sameValue(closed, true);

closed = false;
assert.throws(TypeError, function() {
  closable.every({});
});
assert.sameValue(closed, true);
