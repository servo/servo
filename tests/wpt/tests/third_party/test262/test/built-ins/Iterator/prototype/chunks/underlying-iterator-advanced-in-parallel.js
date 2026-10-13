// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Underlying iterator is advanced after calling chunks
info: |
  Iterator.prototype.chunks ( chunkSize )

features: [iterator-chunking, generators]
includes: [compareArray.js]
---*/
let iterator = (function* () {
  for (let i = 0; i < 6; ++i) {
    yield i;
  }
})();

let chunked = iterator.chunks(2);

let result = chunked.next();
assert.compareArray(result.value, [0, 1]);
assert.sameValue(result.done, false);

let { value, done } = iterator.next();
assert.sameValue(value, 2);
assert.sameValue(done, false);

result = chunked.next();
assert.compareArray(result.value, [3, 4]);
assert.sameValue(result.done, false);

result = chunked.next();
assert.compareArray(result.value, [5]);
assert.sameValue(result.done, false);

result = chunked.next();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
