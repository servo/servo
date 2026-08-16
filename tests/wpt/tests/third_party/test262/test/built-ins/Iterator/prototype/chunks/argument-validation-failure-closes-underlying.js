// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Underlying iterator is closed when chunkSize validation fails
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

let closed = false;
let closable = {
  __proto__: Iterator.prototype,
  get next() {
    throw new Test262Error('next should not be read');
  },
  return() {
    closed = true;
    return {};
  },
};

assert.throws(TypeError, function () {
  closable.chunks();
});
assert.sameValue(closed, true, 'iterator closed when chunkSize is undefined');

closed = false;
assert.throws(RangeError, function () {
  closable.chunks(0);
});
assert.sameValue(closed, true, 'iterator closed when chunkSize is 0');

closed = false;
assert.throws(TypeError, function () {
  closable.chunks(NaN);
});
assert.sameValue(closed, true, 'iterator closed when chunkSize is NaN');

closed = false;
assert.throws(TypeError, function () {
  closable.chunks('1');
});
assert.sameValue(closed, true, 'iterator closed when chunkSize is a string');
