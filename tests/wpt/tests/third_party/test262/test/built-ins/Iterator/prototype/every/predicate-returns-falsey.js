// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.every
description: >
  Iterator.prototype.every returns false and closes the iterator when the predicate returns falsey immediately
info: |
  %Iterator.prototype%.every ( predicate )

  4.f. If ToBoolean(result) is false, return ? IteratorClose(iterated, NormalCompletion(false)).

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 0;
  yield 1;
}

let iter = g();

let predicateCalls = 0;
let result = iter.every(v => {
  ++predicateCalls;
  return false;
});

assert.sameValue(result, false);
assert.sameValue(predicateCalls, 1);

let { done, value } = iter.next();
assert.sameValue(done, true);
assert.sameValue(value, undefined);
