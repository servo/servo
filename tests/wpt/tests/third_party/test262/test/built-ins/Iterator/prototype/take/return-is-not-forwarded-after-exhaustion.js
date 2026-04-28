// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Underlying iterator return is not called after result iterator observes that underlying iterator is exhausted
info: |
  %Iterator.prototype%.take ( limit )

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

let iterator = new TestIterator().take(0);
assert.throws(Test262Error, function () {
  iterator.return();
});
iterator.next();
iterator.return();

iterator = new TestIterator().take(1);
iterator.next();
iterator.return();

iterator = new TestIterator().take(1);
assert.throws(Test262Error, function () {
  iterator.return();
});
iterator.next();
iterator.return();

iterator = new TestIterator().take(1).take(1).take(1).take(1).take(1);
assert.throws(Test262Error, function () {
  iterator.return();
});
iterator.next();
iterator.return();
