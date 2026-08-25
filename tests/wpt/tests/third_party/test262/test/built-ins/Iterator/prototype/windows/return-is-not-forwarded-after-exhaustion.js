// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Underlying iterator return is not called after result iterator observes
  that underlying iterator is exhausted
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

features: [iterator-chunking, class]
---*/

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

let iterator = new TestIterator().windows(1);
assert.throws(Test262Error, function () {
  iterator.return();
});
iterator.next();
iterator.return();

iterator = new TestIterator().windows(1);
iterator.next();
iterator.return();
