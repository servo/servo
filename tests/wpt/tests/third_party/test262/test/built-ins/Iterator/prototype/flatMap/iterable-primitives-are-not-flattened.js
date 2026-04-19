// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Iterator.prototype.flatMap does not respect the iterability of any primitive
info: |
  %Iterator.prototype%.flatMap ( mapper )

  5.b.vi. Let innerIterator be Completion(GetIteratorFlattenable(mapped)).

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/

function* g() {
  yield 0;
}

Number.prototype[Symbol.iterator] = function* () {
  let i = 0;
  let target = this >>> 0;
  while (i < target) {
    yield i;
    ++i;
  }
};

assert.compareArray(Array.from(5), [0, 1, 2, 3, 4]);

assert.throws(TypeError, function () {
  for (let unused of g().flatMap(v => 5));
});

let iter = g().flatMap(v => new Number(5));
assert.compareArray(Array.from(iter), [0, 1, 2, 3, 4]);
