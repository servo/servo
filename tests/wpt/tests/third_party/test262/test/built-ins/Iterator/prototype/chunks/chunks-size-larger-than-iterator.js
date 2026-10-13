// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  When chunkSize is larger than the number of elements, a single partial chunk
  is yielded
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
  yield 5;
}

let chunks = Array.from(g().chunks(100));

assert.sameValue(chunks.length, 1);
assert.compareArray(chunks[0], [0, 1, 2, 3, 4, 5]);
