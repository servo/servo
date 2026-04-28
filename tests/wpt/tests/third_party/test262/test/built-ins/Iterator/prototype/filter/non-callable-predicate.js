// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.filter
description: >
  Iterator.prototype.filter expects to be called with a callable argument.
info: |
  %Iterator.prototype%.filter ( predicate )

  2. If IsCallable(predicate) is false, throw a TypeError exception.

features: [iterator-helpers]
flags: []
---*/
let nonCallable = {};
let iterator = (function* () {})();

assert.throws(TypeError, function () {
  iterator.filter(nonCallable);
});
