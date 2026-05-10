// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Iterator.prototype.reduce calls reducer once for each value yielded by the underlying iterator when passed an initial value
info: |
  %Iterator.prototype%.reduce ( reducer )

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 'a';
}

let iter = g();

let assertionCount = 0;
let initialValue = {};
let result = iter.reduce((memo, v, count) => {
  switch (v) {
    case 'a':
      assert.sameValue(memo, initialValue);
      assert.sameValue(count, 0);
      break;
    default:
      throw new Error();
  }
  ++assertionCount;
  return v;
}, initialValue);

assert.sameValue(result, 'a');
assert.sameValue(assertionCount, 1);
