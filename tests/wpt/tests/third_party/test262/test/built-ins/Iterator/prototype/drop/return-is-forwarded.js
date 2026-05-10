// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  Underlying iterator return is called when result iterator is closed
info: |
  %Iterator.prototype%.drop ( limit )

  6.c.iii. Let completion be Completion(Yield(? IteratorValue(next))).
  6.c.iv. IfAbruptCloseIterator(completion, iterated).

features: [iterator-helpers]
flags: []
---*/
let returnCount = 0;

class TestIterator extends Iterator {
  next() {
    return {
      done: false,
      value: 1,
    };
  }
  return() {
    ++returnCount;
    return {};
  }
}

let iterator = new TestIterator().drop(0);
assert.sameValue(returnCount, 0);
iterator.return();
assert.sameValue(returnCount, 1);
iterator.return();
assert.sameValue(returnCount, 1);

returnCount = 0;

iterator = new TestIterator().drop(1);
assert.sameValue(returnCount, 0);
iterator.return();
assert.sameValue(returnCount, 1);
iterator.return();
assert.sameValue(returnCount, 1);

returnCount = 0;

iterator = new TestIterator().drop(1).drop(1).drop(1).drop(1).drop(1);
assert.sameValue(returnCount, 0);
iterator.return();
assert.sameValue(returnCount, 1);
iterator.return();
assert.sameValue(returnCount, 1);
