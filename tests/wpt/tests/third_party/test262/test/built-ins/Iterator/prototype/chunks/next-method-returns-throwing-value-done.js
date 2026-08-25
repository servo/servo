// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Underlying iterator next returns object with throwing value getter, but is
  already done
info: |
  Iterator.prototype.chunks ( chunkSize )

features: [iterator-chunking, class]
---*/
class ThrowingIterator extends Iterator {
  next() {
    return {
      done: true,
      get value() {
        throw new Test262Error();
      }
    };
  }
  get return() {
    throw new Test262Error();
  }
}

let iterator = new ThrowingIterator().chunks(1);
iterator.next();
