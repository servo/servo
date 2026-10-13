// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Throws when getting the next method from the underlying iterator throws
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  7. Set iterated to ? GetIteratorDirect(O).

features: [iterator-chunking, class]
---*/
class ThrowingIterator extends Iterator {
  get next() {
    throw new Test262Error();
  }
  get return() {
    throw new TypeError();
  }
}

let iter = new ThrowingIterator();

assert.throws(Test262Error, function () {
  iter.windows(1);
});
