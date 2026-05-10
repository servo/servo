// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Iterator.prototype.flatMap does not respect the iterability of primitive strings
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

assert.throws(TypeError, function () {
  for (let unused of g().flatMap(v => 'string'));
});

let iter = g().flatMap(v => new String('string'));
assert.compareArray(Array.from(iter), ['s', 't', 'r', 'i', 'n', 'g']);
