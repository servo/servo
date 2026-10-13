// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Iterator.prototype.chunks yields no chunks when the iterator is already
  exhausted
info: |
  Iterator.prototype.chunks ( chunkSize )

  6.a.i. Let value be ? IteratorStepValue(iterated).
  6.a.ii. If value is ~done~, then
    6.a.ii.a. If buffer is not empty, then
      ...
    6.a.ii.b. Return ReturnCompletion(undefined).

features: [iterator-chunking, generators]
---*/
function* g() {}

let chunks = Array.from(g().chunks(2));
assert.sameValue(chunks.length, 0);
