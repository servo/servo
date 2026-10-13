// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Iterator.prototype.windows throws RangeError when windowSize is an integral
  Number outside the valid range [1, 2^32 - 1]
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  6. If windowSize is not in the inclusive interval from 1𝔽 to 𝔽(2^32 - 1), then
    a. Let error be ThrowCompletion(a newly created RangeError object).
    b. Return ? IteratorClose(iterated, error).

features: [iterator-chunking, generators]
---*/
let iterator = (function* () {})();

assert.throws(RangeError, () => {
  iterator.windows(0);
});

assert.throws(RangeError, () => {
  iterator.windows(-0);
});

assert.throws(RangeError, () => {
  iterator.windows(-1);
});

assert.throws(RangeError, () => {
  iterator.windows(2 ** 32);
});

assert.throws(RangeError, () => {
  iterator.windows(2 ** 53);
});

// Boundary: valid values do not throw
iterator.windows(1);
iterator.windows(2 ** 32 - 1);
