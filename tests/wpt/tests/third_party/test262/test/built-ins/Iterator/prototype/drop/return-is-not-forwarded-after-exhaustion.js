// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  Underlying iterator return is not called after result iterator observes that underlying iterator is exhausted
info: |
  %Iterator.prototype%.drop ( limit )

features: [iterator-helpers]
flags: []
---*/
let returnCount = 0;

class TestIterator extends Iterator {
  next() {
    return {
      done: true,
      value: undefined,
    };
  }
  return() {
    throw new Test262Error();
  }
}

let iterator = new TestIterator().drop(0);
assert.throws(Test262Error, function () {
  iterator.return();
});
iterator.next();
iterator.return();

iterator = new TestIterator().drop(1);
iterator.next();
iterator.return();

iterator = new TestIterator().drop(1);
assert.throws(Test262Error, function () {
  iterator.return();
});
iterator.next();
iterator.return();

iterator = new TestIterator().drop(1).drop(1).drop(1).drop(1).drop(1);
assert.throws(Test262Error, function () {
  iterator.return();
});
iterator.next();
iterator.return();
