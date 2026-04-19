// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.find
description: >
  Iterator.prototype.find returns the found value and closes the iterator when the predicate returns falsey for some iterated values and truthy for others
info: |
  %Iterator.prototype%.find ( predicate )

features: [iterator-helpers]
flags: []
---*/
function* g() {
  for (let i = 0; i < 5; ++i) {
    yield i;
  }
}

let iter = g();

let predicateCalls = 0;
let result = iter.find(v => {
  ++predicateCalls;
  return v > 2;
});

assert.sameValue(result, 3);
assert.sameValue(predicateCalls, 4);

let { done, value } = iter.next();
assert.sameValue(done, true);
assert.sameValue(value, undefined);
