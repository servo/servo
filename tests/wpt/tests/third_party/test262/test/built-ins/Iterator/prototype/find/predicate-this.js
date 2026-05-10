// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.find
description: >
  Iterator.prototype.find predicate this value is undefined
info: |
  %Iterator.prototype%.find ( predicate )

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 0;
}

let iter = g();

let expectedThis = function () {
  return this;
}.call(undefined);

let assertionCount = 0;
let result = iter.find(function (v, count) {
  assert.sameValue(this, expectedThis);
  ++assertionCount;
  return true;
});

assert.sameValue(result, 0);
assert.sameValue(assertionCount, 1);
