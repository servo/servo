// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.every
description: >
  Iterator.prototype.every returns false and closes the iterator when the predicate returns truthy for some iterated values and falsey for others
info: |
  %Iterator.prototype%.every ( predicate )

  4.f. If ToBoolean(result) is false, return ? IteratorClose(iterated, NormalCompletion(false)).

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
let result = iter.every(v => {
  ++predicateCalls;
  return v < 3;
});

assert.sameValue(result, false);
assert.sameValue(predicateCalls, 4);

let { done, value } = iter.next();
assert.sameValue(done, true);
assert.sameValue(value, undefined);
