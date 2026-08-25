// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Iterator.prototype.reduce reducer this value is undefined
info: |
  %Iterator.prototype%.reduce ( reducer )

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 0;
  yield 1;
}

let iter = g();

let expectedThis = function () {
  return this;
}.call(undefined);

let assertionCount = 0;
let result = iter.reduce(function (memo, v, count) {
  assert.sameValue(this, expectedThis);
  ++assertionCount;
  return memo;
});

assert.sameValue(result, 0);
assert.sameValue(assertionCount, 1);
