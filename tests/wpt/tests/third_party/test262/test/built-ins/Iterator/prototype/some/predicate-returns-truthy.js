// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  Iterator.prototype.some returns true and closes the iterator when the predicate returns truthy immediately
info: |
  %Iterator.prototype%.some ( predicate )

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 0;
  yield 1;
  yield 2;
}

let iter = g();

let predicateCalls = 0;
let result = iter.some(v => {
  ++predicateCalls;
  return true;
});

assert.sameValue(result, true);
assert.sameValue(predicateCalls, 1);

let { done, value } = iter.next();
assert.sameValue(done, true);
assert.sameValue(value, undefined);
