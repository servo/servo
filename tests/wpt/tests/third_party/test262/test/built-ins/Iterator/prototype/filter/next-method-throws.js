// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.filter
description: >
  Underlying iterator has throwing next method
info: |
  %Iterator.prototype%.filter ( predicate )

  3.b.i. Let next be ? IteratorStep(iterated).

features: [iterator-helpers]
flags: []
---*/
class ThrowingIterator extends Iterator {
  next() {
    throw new Test262Error();
  }
}

let iterator = new ThrowingIterator().filter(() => true);

assert.throws(Test262Error, function () {
  iterator.next();
});
