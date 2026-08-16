// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  When chunkSize is 1, each element is yielded as a single-element array
info: |
  Iterator.prototype.chunks ( chunkSize )

features: [iterator-chunking, generators]
includes: [compareArray.js]
---*/
function* g() {
  yield 0;
  yield 1;
  yield 2;
  yield 3;
  yield 4;
}

let chunks = Array.from(g().chunks(1));

assert.sameValue(chunks.length, 5);
assert.compareArray(chunks[0], [0]);
assert.compareArray(chunks[1], [1]);
assert.compareArray(chunks[2], [2]);
assert.compareArray(chunks[3], [3]);
assert.compareArray(chunks[4], [4]);
