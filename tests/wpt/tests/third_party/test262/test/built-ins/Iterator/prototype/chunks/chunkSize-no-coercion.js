// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Iterator.prototype.chunks does not coerce chunkSize using ToNumber; the
  argument must already be a Number. Unlike take/drop, valueOf and toString
  are never called.
info: |
  Iterator.prototype.chunks ( chunkSize )

  4. If chunkSize is not a Number, throw a TypeError exception.

features: [iterator-chunking, generators]
---*/
let iterator = (function* () {})();

let valueOfCalled = false;
assert.throws(TypeError, () => {
  iterator.chunks({
    valueOf() {
      valueOfCalled = true;
      return 2;
    }
  });
});
assert.sameValue(valueOfCalled, false, 'valueOf must not be called');

let toStringCalled = false;
assert.throws(TypeError, () => {
  iterator.chunks({
    toString() {
      toStringCalled = true;
      return '2';
    }
  });
});
assert.sameValue(toStringCalled, false, 'toString must not be called');
