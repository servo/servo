// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Iterator.prototype.reduce reducer is passed the yielded value and a counter as arguments
info: |
  %Iterator.prototype%.reduce ( reducer )

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 'a';
  yield 'b';
  yield 'c';
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
    case 'b':
      assert.sameValue(memo, 'a');
      assert.sameValue(count, 1);
      break;
    case 'c':
      assert.sameValue(memo, 'b');
      assert.sameValue(count, 2);
      break;
    default:
      throw new Error();
  }
  ++assertionCount;
  return v;
}, initialValue);

assert.sameValue(result, 'c');
assert.sameValue(assertionCount, 3);
