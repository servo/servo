// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Iterator.prototype.windows throws TypeError when undersized is not a valid
  value
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  5. If undersized is undefined, set undersized to "only-full".
  6. If undersized is neither "only-full" nor "allow-partial", then
    a. Let error be ThrowCompletion(a newly created TypeError object).
    b. Return ? IteratorClose(iterated, error).

features: [iterator-chunking, generators]
---*/
let iterator = (function* () {})();

assert.throws(TypeError, () => {
  iterator.windows(1, null);
});

assert.throws(TypeError, () => {
  iterator.windows(1, '');
});

assert.throws(TypeError, () => {
  iterator.windows(1, 'something else');
});

assert.throws(TypeError, () => {
  iterator.windows(1, 0);
});

assert.throws(TypeError, () => {
  iterator.windows(1, true);
});

assert.throws(TypeError, () => {
  iterator.windows(1, false);
});

assert.throws(TypeError, () => {
  iterator.windows(1, {});
});

assert.throws(TypeError, () => {
  iterator.windows(1, Symbol());
});
