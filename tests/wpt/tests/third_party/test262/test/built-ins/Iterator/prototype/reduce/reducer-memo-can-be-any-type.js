// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Iterator.prototype.reduce reducer can return any ECMAScript language value
info: |
  %Iterator.prototype%.reduce ( reducer )

features: [iterator-helpers]
flags: []
---*/

const values = [undefined, null, true, false, 0, -0, 1, NaN, Infinity, "string", Symbol(), 0n, {}, [], () => {}];

let iter = values[Symbol.iterator]();

let assertionCount = 0;
let initialValue = {};
let result = iter.reduce((memo, v, count) => {
  if (count == 0) {
    assert.sameValue(memo, initialValue);
  } else {
    assert.sameValue(memo, values[count - 1]);
  }
  ++assertionCount;
  return v;
}, initialValue);

assert.sameValue(result, values[values.length - 1]);
assert.sameValue(assertionCount, values.length);
