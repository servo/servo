// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Underlying iterator is closed when argument validation fails
info: |
  %Iterator.prototype%.take ( limit )

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

assert.throws(RangeError, function() {
  closable.take();
});
assert.sameValue(closed, true);

closed = false;
assert.throws(RangeError, function() {
  closable.take(NaN);
});
assert.sameValue(closed, true);

closed = false;
assert.throws(RangeError, function() {
  closable.take(-1);
});
assert.sameValue(closed, true);

closed = false;
class ShouldNotGetValueOf {}
assert.throws(ShouldNotGetValueOf, function() {
  closable.take({ get valueOf() { throw new ShouldNotGetValueOf(); }});
});
assert.sameValue(closed, true);
