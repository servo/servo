// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  Underlying iterator return is not called when result iterator is exhausted
info: |
  %Iterator.prototype%.drop ( limit )

    6.b.ii. Let next be ? IteratorStep(iterated).
    6.b.iii. If next is false, return undefined.
  6.c. Repeat,
    6.c.i. Let next be ? IteratorStep(iterated).
    6.c.ii. If next is false, return undefined.

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 0;
  yield 1;
  yield 2;
}

class TestIterator extends Iterator {
  get next() {
    let n = g();
    return function() {
      return n.next();
    };
  }
  return() {
    throw new Test262Error();
  }
}

let iterator = new TestIterator();
iterator = iterator.drop(0);
iterator.next();
iterator.next();
iterator.next();
iterator.next();
iterator.next();

iterator = new TestIterator();
iterator = iterator.drop(1);
iterator.next();
iterator.next();
iterator.next();
iterator.next();

iterator = new TestIterator();
iterator = iterator.drop(1).drop(1).drop(1).drop(1).drop(1);
iterator.next();
iterator.next();

iterator = new TestIterator();
iterator = iterator.drop(10);
iterator.next();
iterator.next();
