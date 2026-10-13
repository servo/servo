// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Underlying iterator is closed after calling chunks
info: |
  Iterator.prototype.chunks ( chunkSize )

features: [iterator-chunking, generators]
---*/
let iterator = (function* () {
  for (let i = 0; i < 5; ++i) {
    yield i;
  }
})();

let chunked = iterator.chunks(2);

iterator.return();

let { value, done } = chunked.next();

assert.sameValue(value, undefined);
assert.sameValue(done, true);
