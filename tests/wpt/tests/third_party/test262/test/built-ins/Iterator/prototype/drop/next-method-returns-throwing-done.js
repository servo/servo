// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  Underlying iterator next returns object with throwing done getter
info: |
  %Iterator.prototype%.drop ( limit )

  6.b.ii. Let next be ? IteratorStep(iterated).

  6.c.i. Let next be ? IteratorStep(iterated).

features: [iterator-helpers]
flags: []
---*/
class ThrowingIterator extends Iterator {
  next() {
    return {
      get done() {
        throw new Test262Error();
      },
      value: 1,
    };
  }
  return() {
    throw new Error();
  }
}

let iterator = new ThrowingIterator().drop(0);

assert.throws(Test262Error, function () {
  iterator.next();
});

iterator = new ThrowingIterator().drop(1);

assert.throws(Test262Error, function () {
  iterator.next();
});
