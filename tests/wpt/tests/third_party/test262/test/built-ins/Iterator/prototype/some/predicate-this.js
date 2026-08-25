// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  Iterator.prototype.some predicate this value is undefined
info: |
  %Iterator.prototype%.some ( predicate )

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
let result = iter.some(function (v, count) {
  assert.sameValue(this, expectedThis);
  ++assertionCount;
  return true;
});

assert.sameValue(result, true);
assert.sameValue(assertionCount, 1);
