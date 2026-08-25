// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.map
description: >
  The values returned by the mapper are the values that are yielded by the iterator returned by map
info: |
  %Iterator.prototype%.map ( mapper )

  5.b.vi. Let completion be Completion(Yield(mapped)).

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/
function* g() {
  for (let i = 0; i < 5; ++i) {
    yield i;
  }
}

assert.compareArray(Array.from(g()), [0, 1, 2, 3, 4]);
assert.compareArray(Array.from(g().map(x => x)), [0, 1, 2, 3, 4]);
assert.compareArray(Array.from(g().map(() => 0)), [0, 0, 0, 0, 0]);
assert.compareArray(
  Array.from(
    g()
      .map(() => 0)
      .map((v, c) => c)
  ),
  [0, 1, 2, 3, 4]
);
assert.compareArray(Array.from(g().map(x => x * 2)), [0, 2, 4, 6, 8]);

let obj = {};
assert.compareArray(Array.from(g().map(() => obj)), [obj, obj, obj, obj, obj]);
