// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.filter
description: >
  Iterator.prototype.filter returns only items for which the predicate returned true.
info: |
  %Iterator.prototype%.filter ( filterer )

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 1;
  yield 0;
  yield 2;
  yield 0;
  yield 3;
  yield 0;
  yield 4;
}

let iterator = g();

let predicateCalls = 0;
iterator = iterator.filter(value => {
  ++predicateCalls;
  return value !== 0;
});

let resultCount = 0;
for (let value of iterator) {
  ++resultCount;
  assert.sameValue(value, resultCount);
}
assert.sameValue(resultCount, 4);

let { value, done } = iterator.next();
assert.sameValue(value, undefined);
assert.sameValue(done, true);
