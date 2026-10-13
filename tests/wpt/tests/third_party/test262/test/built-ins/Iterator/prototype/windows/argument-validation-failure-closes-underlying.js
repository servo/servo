// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Underlying iterator is closed when argument validation fails
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  3. Let iterated be the Iterator Record { [[Iterator]]: O, [[NextMethod]]: undefined, [[Done]]: false }.
  4. If windowSize is not a Number, throw a TypeError exception ... IteratorClose(iterated, error).
  5. If windowSize is not an integral Number, throw a TypeError exception ... IteratorClose(iterated, error).
  6. If windowSize is not in the inclusive interval from 1𝔽 to 𝔽(2^32 - 1), then
    a. Let error be ThrowCompletion(a newly created RangeError object).
    b. Return ? IteratorClose(iterated, error).
  ...
  8. If undersized is neither "only-full" nor "allow-partial", then
    a. Let error be ThrowCompletion(a newly created TypeError object).
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
  }
};

// windowSize validation failure closes
assert.throws(TypeError, function () {
  closable.windows();
});
assert.sameValue(closed, true, 'iterator closed when windowSize is undefined');

closed = false;
assert.throws(RangeError, function () {
  closable.windows(0);
});
assert.sameValue(closed, true, 'iterator closed when windowSize is 0');

closed = false;
assert.throws(TypeError, function () {
  closable.windows(NaN);
});
assert.sameValue(closed, true, 'iterator closed when windowSize is NaN');

closed = false;
assert.throws(TypeError, function () {
  closable.windows('1');
});
assert.sameValue(closed, true, 'iterator closed when windowSize is a string');

// undersized validation failure closes
closed = false;
assert.throws(TypeError, function () {
  closable.windows(1, null);
});
assert.sameValue(closed, true, 'iterator closed when undersized is null');

closed = false;
assert.throws(TypeError, function () {
  closable.windows(1, 'bad');
});
assert.sameValue(closed, true, 'iterator closed when undersized is invalid string');
