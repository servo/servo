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
let result = iter.reduce((memo, v, count) => {
  switch (v) {
    case 'b':
      assert.sameValue(memo, 'a');
      assert.sameValue(count, 1);
      break;
    case 'c':
      assert.sameValue(memo, 'b');
      assert.sameValue(count, 2);
      break;
    case 'a':
    default:
      throw new Error();
  }
  ++assertionCount;
  return v;
});

assert.sameValue(result, 'c');
assert.sameValue(assertionCount, 2);
