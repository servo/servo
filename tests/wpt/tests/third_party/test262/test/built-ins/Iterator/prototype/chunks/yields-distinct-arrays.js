// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Each yielded chunk is a distinct new Array object
info: |
  Iterator.prototype.chunks ( chunkSize )

  6.a.iv.a. Let completion be Completion(Yield(CreateArrayFromList(buffer))).

features: [iterator-chunking, generators]
---*/
function* g() {
  yield 0;
  yield 1;
  yield 2;
  yield 3;
}

let chunks = Array.from(g().chunks(2));

assert.sameValue(chunks.length, 2);
assert(Array.isArray(chunks[0]), 'chunks[0] is an Array');
assert(Array.isArray(chunks[1]), 'chunks[1] is an Array');
assert.notSameValue(chunks[0], chunks[1], 'each chunk is a distinct array');
