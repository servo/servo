// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.forEach
description: >
  Iterator.prototype.forEach fn this value is undefined
info: |
  %Iterator.prototype%.forEach ( fn )

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
let result = iter.forEach(function (v, count) {
  assert.sameValue(this, expectedThis);
  ++assertionCount;
});

assert.sameValue(result, undefined);
assert.sameValue(assertionCount, 1);
