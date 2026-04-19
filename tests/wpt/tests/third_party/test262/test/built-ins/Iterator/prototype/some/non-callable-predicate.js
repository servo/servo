// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  Iterator.prototype.some expects to be called with a callable argument.
info: |
  %Iterator.prototype%.some ( predicate )

features: [iterator-helpers]
flags: []
---*/
let nonCallable = {};
let iterator = (function* () {
  yield 1;
})();

assert.throws(TypeError, function () {
  iterator.some(nonCallable);
});
