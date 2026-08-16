// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  All chunks are full-sized when the iterator length is evenly divisible by
  chunkSize
info: |
  Iterator.prototype.chunks ( chunkSize )

  6.a.iv. If the number of elements in buffer is ℝ(chunkSize), then
    6.a.iv.a. Let completion be Completion(Yield(CreateArrayFromList(buffer))).

features: [iterator-chunking, generators]
includes: [compareArray.js]
---*/
function* g(n) {
  for (let i = 0; i < n; ++i) {
    yield i;
  }
}

let chunks = Array.from(g(4).chunks(2));

assert.sameValue(chunks.length, 2);
assert.compareArray(chunks[0], [0, 1]);
assert.compareArray(chunks[1], [2, 3]);

chunks = Array.from(g(6).chunks(3));

assert.sameValue(chunks.length, 2);
assert.compareArray(chunks[0], [0, 1, 2]);
assert.compareArray(chunks[1], [3, 4, 5]);
