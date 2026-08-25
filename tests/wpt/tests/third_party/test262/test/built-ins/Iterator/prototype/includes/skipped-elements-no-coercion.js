// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  skippedElements is not coerced; non-Number values throw before valueOf/toString
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

let valueOfCalled = false;
assert.throws(TypeError, function() {
  iterator.includes(0, {
    valueOf() {
      valueOfCalled = true;
      return 0;
    },
  });
});
assert.sameValue(valueOfCalled, false, 'valueOf must not be called');
assert.sameValue(closed, true, 'iterator closed when skippedElements validation fails');

closed = false;
let toStringCalled = false;
assert.throws(TypeError, function() {
  iterator.includes(0, {
    toString() {
      toStringCalled = true;
      return '0';
    },
  });
});
assert.sameValue(toStringCalled, false, 'toString must not be called');
assert.sameValue(closed, true, 'iterator closed when skippedElements validation fails');
