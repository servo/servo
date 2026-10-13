// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Original validation error is preserved when closing the underlying iterator
  throws
info: |
  Iterator.prototype.chunks ( chunkSize )

  3. Let iterated be the Iterator Record { [[Iterator]]: O, [[NextMethod]]: undefined, [[Done]]: false }.
  4. If chunkSize is not a Number, throw a TypeError exception ... IteratorClose(iterated, error).
  5. If chunkSize is not an integral Number, throw a TypeError exception ... IteratorClose(iterated, error).
  6. If chunkSize is not in the inclusive interval from 1𝔽 to 𝔽(2^32 - 1), then
    a. Let error be ThrowCompletion(a newly created RangeError object).
    b. Return ? IteratorClose(iterated, error).

features: [iterator-chunking]
---*/

let returnGets = 0;
let closable = {
  __proto__: Iterator.prototype,
  get next() {
    throw new Test262Error('next should not be read');
  },
  get return() {
    ++returnGets;
    throw new Test262Error('return getter error should be masked');
  },
};

assert.throws(RangeError, function () {
  closable.chunks(0);
});
assert.sameValue(returnGets, 1, 'return getter is still consulted');
