// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Last chunk may be smaller than chunkSize when the iterator is not evenly
  divisible
info: |
  Iterator.prototype.chunks ( chunkSize )

  6.a.i. Let value be ? IteratorStepValue(iterated).
  6.a.ii. If value is ~done~, then
    6.a.ii.a. If buffer is not empty, then
      6.a.ii.a.i. Perform Completion(Yield(CreateArrayFromList(buffer))).

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

let chunks = Array.from(g().chunks(2));

assert.sameValue(chunks.length, 3);
assert.compareArray(chunks[0], [0, 1]);
assert.compareArray(chunks[1], [2, 3]);
assert.compareArray(chunks[2], [4]);
