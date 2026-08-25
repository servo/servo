// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  Iterator.from supports non-iterable iterators
info: |
  Iterator.from ( O )

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

let n = g();
let iter = {
  next() {
    return n.next();
  },
};

assert.compareArray(Array.from(Iterator.from(iter)), [0, 1, 2, 3]);
