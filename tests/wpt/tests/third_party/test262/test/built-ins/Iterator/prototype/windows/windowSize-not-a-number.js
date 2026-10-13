// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Iterator.prototype.windows throws TypeError when windowSize is not an
  integral Number
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  4. If windowSize is not a Number, throw a TypeError exception.
  5. If windowSize is not an integral Number, throw a TypeError exception.

features: [iterator-chunking, generators]
---*/
let iterator = (function* () {})();

assert.throws(TypeError, () => {
  iterator.windows();
});

assert.throws(TypeError, () => {
  iterator.windows(undefined);
});

assert.throws(TypeError, () => {
  iterator.windows('1');
});

assert.throws(TypeError, () => {
  iterator.windows(true);
});

assert.throws(TypeError, () => {
  iterator.windows(null);
});

assert.throws(TypeError, () => {
  iterator.windows({});
});

assert.throws(TypeError, () => {
  iterator.windows(Symbol());
});

assert.throws(TypeError, () => {
  iterator.windows([2]);
});

assert.throws(TypeError, () => {
  iterator.windows(NaN);
});

assert.throws(TypeError, () => {
  iterator.windows(0.5);
});

assert.throws(TypeError, () => {
  iterator.windows(1.5);
});

assert.throws(TypeError, () => {
  iterator.windows(Infinity);
});

assert.throws(TypeError, () => {
  iterator.windows(-Infinity);
});
