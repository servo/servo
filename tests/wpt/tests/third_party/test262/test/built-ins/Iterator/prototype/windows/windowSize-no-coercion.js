// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Iterator.prototype.windows does not coerce windowSize using ToNumber; the
  argument must already be a Number. Unlike take/drop, valueOf and toString
  are never called.
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  4. If windowSize is not a Number, throw a TypeError exception.

features: [iterator-chunking, generators]
---*/
let iterator = (function* () {})();

let valueOfCalled = false;
assert.throws(TypeError, () => {
  iterator.windows({
    valueOf() {
      valueOfCalled = true;
      return 2;
    }
  });
});
assert.sameValue(valueOfCalled, false, 'valueOf must not be called');

let toStringCalled = false;
assert.throws(TypeError, () => {
  iterator.windows({
    toString() {
      toStringCalled = true;
      return '2';
    }
  });
});
assert.sameValue(toStringCalled, false, 'toString must not be called');
