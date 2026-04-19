// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Iterator.prototype.flatMap flattens non-iterable iterators returned by the mapper
info: |
  %Iterator.prototype%.flatMap ( mapper )

  5.b.vi. Let innerIterator be Completion(GetIteratorFlattenable(mapped)).

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
  let i = 0;
  return {
    next: function () {
      if (i < v) {
        ++i;
        return {
          value: v,
          done: false,
        };
      } else {
        return {
          value: undefined,
          done: true,
        };
      }
    },
  };
});

assert.compareArray(Array.from(iter), [1, 2, 2, 3, 3, 3]);
