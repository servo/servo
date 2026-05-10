// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  Iterator.from falls back to treating its parameter as an iterator if the Symbol.iterator property is null/undefined
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
}

let iter = (function () {
  let n = g();
  return {
    [Symbol.iterator]: 0,
    next: () => n.next(),
  };
})();

assert.throws(TypeError, function () {
  Iterator.from(iter);
});

iter = (function () {
  let n = g();
  return {
    [Symbol.iterator]: null,
    next: () => n.next(),
  };
})();

assert.compareArray(Array.from(Iterator.from(iter)), [0, 1, 2]);

iter = (function () {
  let n = g();
  return {
    [Symbol.iterator]: undefined,
    next: () => n.next(),
  };
})();

assert.compareArray(Array.from(Iterator.from(iter)), [0, 1, 2]);
