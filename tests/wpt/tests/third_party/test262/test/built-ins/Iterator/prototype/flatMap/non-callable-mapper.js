// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Iterator.prototype.flatMap expects to be called with a callable argument.
info: |
  %Iterator.prototype%.flatMap ( mapper )

  3. If IsCallable(mapper) is false, throw a TypeError exception.

features: [iterator-helpers]
flags: []
---*/
let nonCallable = {};
let iterator = (function* () {})();

assert.throws(TypeError, function () {
  iterator.flatMap(nonCallable);
});
