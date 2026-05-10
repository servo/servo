// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Iterator.prototype.flatMap flattens iterables returned by the mapper
info: |
  %Iterator.prototype%.flatMap ( mapper )

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/

function* g() {
  yield 0;
  yield 1;
  yield 2;
  yield 3;
}

let iter = g().flatMap((v, count) => {
  let result = [];
  for (let i = 0; i < v; ++i) {
    result.push(v);
  }
  return result;
});

assert.compareArray(Array.from(iter), [1, 2, 2, 3, 3, 3]);
