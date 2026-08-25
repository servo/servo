// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Iterator.prototype.chunks throws RangeError when chunkSize is an integral
  Number outside the valid range [1, 2^32 - 1]
info: |
  Iterator.prototype.chunks ( chunkSize )

  6. If chunkSize is not in the inclusive interval from 1𝔽 to 𝔽(2^32 - 1), then
    a. Let error be ThrowCompletion(a newly created RangeError object).
    b. Return ? IteratorClose(iterated, error).

features: [iterator-chunking, generators, exponentiation]
---*/
let iterator = (function* () {})();

assert.throws(RangeError, () => {
  iterator.chunks(0);
});

assert.throws(RangeError, () => {
  iterator.chunks(-0);
});

assert.throws(RangeError, () => {
  iterator.chunks(-1);
});

assert.throws(RangeError, () => {
  iterator.chunks(2 ** 32);
});

assert.throws(RangeError, () => {
  iterator.chunks(2 ** 53);
});

// Boundary: valid values do not throw
iterator.chunks(1);
iterator.chunks(2 ** 32 - 1);
