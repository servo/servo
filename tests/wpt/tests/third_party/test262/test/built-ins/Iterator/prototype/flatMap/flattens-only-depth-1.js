// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Iterator.prototype.flatMap does not flatten recursively
info: |
  %Iterator.prototype%.flatMap ( mapper )

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/

let arr = [
  {
    [Symbol.iterator]: function () {
      throw new Test262Error();
    },
  },
  {
    next: function () {
      throw new Test262Error();
    },
  },
];

function* g() {
  yield arr;
}

let iter = g().flatMap(v => v);

assert.compareArray(Array.from(iter), arr);
