// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Underlying iterator next throws
info: |
  Iterator.prototype.chunks ( chunkSize )

features: [iterator-chunking, class]
---*/
class ThrowingIterator extends Iterator {
  next() {
    throw new Test262Error();
  }
  get return() {
    throw new TypeError();
  }
}

let iterator = new ThrowingIterator().chunks(1);

assert.throws(Test262Error, function () {
  iterator.next();
});
