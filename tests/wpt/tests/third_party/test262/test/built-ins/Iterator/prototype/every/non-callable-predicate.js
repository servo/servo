// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.every
description: >
  Iterator.prototype.every expects to be called with a callable argument.
info: |
  %Iterator.prototype%.every ( predicate )

  2. If IsCallable(predicate) is false, throw a TypeError exception.

features: [iterator-helpers]
flags: []
---*/
let nonCallable = {};
let iterator = (function* () {
  yield 1;
})();

assert.throws(TypeError, function () {
  iterator.every(nonCallable);
});
