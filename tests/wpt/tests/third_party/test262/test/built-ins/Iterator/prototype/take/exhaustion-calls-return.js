// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Underlying iterator return is called when result iterator is exhausted
info: |
  %Iterator.prototype%.take ( limit )

  8.b.i. If remaining is 0, then
    8.b.i.1. Return ? IteratorClose(iterated, NormalCompletion(undefined)).

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
iterator = iterator.take(0);
assert.throws(Test262Error, function () {
  iterator.next();
});
iterator.next();
iterator.next();

iterator = new TestIterator();
iterator = iterator.take(1);
iterator.next();
assert.throws(Test262Error, function () {
  iterator.next();
});
iterator.next();
iterator.next();

iterator = new TestIterator();
iterator = iterator.take(1).take(1).take(1).take(1).take(1);
iterator.next();
assert.throws(Test262Error, function () {
  iterator.next();
});
iterator.next();
iterator.next();

iterator = new TestIterator();
iterator = iterator.take(5);
iterator.next();
iterator.next();
iterator.next();
iterator.next();
iterator.next();
iterator.next();
iterator.next();
