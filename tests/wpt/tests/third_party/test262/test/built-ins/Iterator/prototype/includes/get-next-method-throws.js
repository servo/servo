// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Iterator has throwing next getter
features: [iterator-includes]
---*/

let iterator = {
  __proto__: Iterator.prototype,
  get next() {
    throw new Test262Error();
  },
  get return() {
    throw new TypeError();
  }
};

assert.throws(Test262Error, function() {
  iterator.includes(0);
});
