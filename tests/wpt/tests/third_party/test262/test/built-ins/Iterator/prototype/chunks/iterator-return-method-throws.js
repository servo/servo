// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Iterator has throwing return
info: |
  Iterator.prototype.chunks ( chunkSize )

features: [iterator-chunking, class]
---*/
class IteratorThrows extends Iterator {
  next() {
    return {
      done: false,
      value: 0,
    };
  }
  return() {
    throw new Test262Error();
  }
}

let iterator = new IteratorThrows().chunks(1);

assert.throws(Test262Error, function () {
  iterator.return();
});
