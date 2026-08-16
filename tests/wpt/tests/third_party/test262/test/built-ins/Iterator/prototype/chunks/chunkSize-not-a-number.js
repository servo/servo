// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Iterator.prototype.chunks throws TypeError when chunkSize is not an integral
  Number
info: |
  Iterator.prototype.chunks ( chunkSize )

  4. If chunkSize is not a Number, throw a TypeError exception.
  5. If chunkSize is not an integral Number, throw a TypeError exception.

features: [iterator-chunking, generators]
---*/
let iterator = (function* () {})();

assert.throws(TypeError, () => {
  iterator.chunks();
});

assert.throws(TypeError, () => {
  iterator.chunks(undefined);
});

assert.throws(TypeError, () => {
  iterator.chunks('1');
});

assert.throws(TypeError, () => {
  iterator.chunks(true);
});

assert.throws(TypeError, () => {
  iterator.chunks(null);
});

assert.throws(TypeError, () => {
  iterator.chunks({});
});

assert.throws(TypeError, () => {
  iterator.chunks(Symbol());
});

assert.throws(TypeError, () => {
  iterator.chunks([2]);
});

assert.throws(TypeError, () => {
  iterator.chunks(NaN);
});

assert.throws(TypeError, () => {
  iterator.chunks(0.5);
});

assert.throws(TypeError, () => {
  iterator.chunks(1.5);
});

assert.throws(TypeError, () => {
  iterator.chunks(Infinity);
});

assert.throws(TypeError, () => {
  iterator.chunks(-Infinity);
});
