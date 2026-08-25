// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  Iterator.prototype.some coerces predicate return value to boolean
info: |
  %Iterator.prototype%.some ( predicate )

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield '';
  yield null;
  yield undefined;
  yield 0;
  yield 1;
  yield 2;
  yield 3;
}

let iter = g();

let predicateCalls = 0;
let result = iter.some(v => {
  ++predicateCalls;
  return v;
});

assert.sameValue(result, true);
assert.sameValue(predicateCalls, 5);
