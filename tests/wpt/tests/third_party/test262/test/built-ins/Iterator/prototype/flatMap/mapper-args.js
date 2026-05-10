// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Iterator.prototype.flatMap mapper is passed the yielded value and a counter as arguments
info: |
  %Iterator.prototype%.flatMap ( mapper )

  5.b.iv. Let mapped be Completion(Call(mapper, undefined, « value, 𝔽(counter) »)).

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 'a';
  yield 'b';
  yield 'c';
  yield 'd';
  yield 'e';
}

let assertionCount = 0;
let iter = g().flatMap((v, count) => {
  switch (v) {
    case 'a':
      assert.sameValue(count, 0);
      ++assertionCount;
      return [0];
    case 'b':
      assert.sameValue(count, 1);
      ++assertionCount;
      return [0];
    case 'c':
      assert.sameValue(count, 2);
      ++assertionCount;
      return [1, 2];
    case 'd':
      assert.sameValue(count, 3);
      ++assertionCount;
      return [3, 4, 5];
    case 'e':
      assert.sameValue(count, 4);
      ++assertionCount;
      return [6, 7, 8, 9];
    default:
      throw new Error();
  }
});

assert.sameValue(assertionCount, 0);

for (let i of iter);

assert.sameValue(assertionCount, 5);
