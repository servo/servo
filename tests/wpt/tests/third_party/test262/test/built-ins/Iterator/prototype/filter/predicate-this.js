// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.filter
description: >
  Iterator.prototype.filter predicate this value is undefined
info: |
  %Iterator.prototype%.filter ( predicate )

  3.b.iv. Let selected be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).

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
iter = iter.filter(function (v, count) {
  assert.sameValue(this, expectedThis);
  ++assertionCount;
  return true;
});

iter.next();
assert.sameValue(assertionCount, 1);
