// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.filter
description: >
  Iterator.prototype.filter predicate is passed the yielded value and a counter as arguments
info: |
  %Iterator.prototype%.filter ( predicate )

  3.b.iv. Let selected be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 'a';
  yield 'b';
  yield 'c';
}

let assertionCount = 0;
let iter = g().filter((v, count) => {
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

assert.sameValue(assertionCount, 0);

for (let i of iter);

assert.sameValue(assertionCount, 3);
