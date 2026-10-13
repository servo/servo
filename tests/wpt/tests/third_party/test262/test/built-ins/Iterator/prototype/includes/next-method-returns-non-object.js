// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Underlying iterator next returns non-object
features: [iterator-includes]
---*/

let iterator = {
  __proto__: Iterator.prototype,
  next() {
    return null;
  },
  get return() {
    throw new Test262Error();
  }
};

assert.throws(TypeError, function() {
  iterator.includes(0);
});
