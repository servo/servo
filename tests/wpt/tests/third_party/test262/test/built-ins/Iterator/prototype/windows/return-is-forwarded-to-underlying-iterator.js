// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Underlying iterator return is called when result iterator is closed
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

features: [iterator-chunking, class]
---*/
let returnCount = 0;

class TestIterator extends Iterator {
  next() {
    return {
      done: false,
      value: 1,
    };
  }
  return() {
    ++returnCount;
    return {};
  }
}

let iterator = new TestIterator().windows(2);
assert.sameValue(returnCount, 0);
iterator.return();
assert.sameValue(returnCount, 1);
iterator.return();
assert.sameValue(returnCount, 1);
