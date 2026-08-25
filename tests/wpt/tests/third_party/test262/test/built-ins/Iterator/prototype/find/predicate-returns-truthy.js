// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.find
description: >
  Iterator.prototype.find returns the found value and closes the iterator when the predicate returns truthy immediately
info: |
  %Iterator.prototype%.find ( predicate )

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
let result = iter.find(v => {
  ++predicateCalls;
  return true;
});

assert.sameValue(result, 0);
assert.sameValue(predicateCalls, 1);

let { done, value } = iter.next();
assert.sameValue(done, true);
assert.sameValue(value, undefined);
