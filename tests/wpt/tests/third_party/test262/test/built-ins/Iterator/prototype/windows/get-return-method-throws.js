// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Underlying iterator return is a throwing getter
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

features: [iterator-chunking, class]
---*/
class TestIterator extends Iterator {
  next() {
    return {
      done: false,
      value: 1,
    };
  }
  get return() {
    throw new Test262Error();
  }
}

let iterator = new TestIterator().windows(1);
iterator.next();

assert.throws(Test262Error, function () {
  iterator.return();
});
