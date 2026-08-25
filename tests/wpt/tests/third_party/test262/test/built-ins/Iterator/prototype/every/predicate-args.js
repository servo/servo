// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.every
description: >
  Iterator.prototype.every predicate is passed the yielded value and a counter as arguments
info: |
  %Iterator.prototype%.every ( predicate )

  4.d. Let result be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 'a';
  yield 'b';
  yield 'c';
}

let iter = g();

let assertionCount = 0;
let result = iter.every((v, count) => {
  switch (v) {
    case 'a':
      assert.sameValue(count, 0);
      break;
    case 'b':
      assert.sameValue(count, 1);
      break;
    case 'c':
      assert.sameValue(count, 2);
      break;
    default:
      throw new Error();
  }
  ++assertionCount;
  return true;
});

assert.sameValue(result, true);
assert.sameValue(assertionCount, 3);
