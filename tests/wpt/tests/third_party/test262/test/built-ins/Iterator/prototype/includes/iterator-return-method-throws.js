// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Iterator has throwing return method
features: [iterator-includes]
---*/

let counter = 0;
let iterator = {
  __proto__: Iterator.prototype,
  next() {
    if (counter === 0) {
      ++counter;
      return { done: false, value: 0 };
    } else {
      return { done: true, value: undefined };
    }
  },
  return() {
    throw new Test262Error();
  }
};

assert.sameValue(iterator.includes(1), false);
counter = 0;

assert.throws(Test262Error, function() {
  iterator.includes(0);
});
