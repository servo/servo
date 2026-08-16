// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Underlying iterator is advanced after calling windows
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

features: [iterator-chunking, generators]
includes: [compareArray.js]
---*/
let iterator = (function* () {
  for (let i = 0; i < 6; ++i) {
    yield i;
  }
})();

let windowed = iterator.windows(2);

let result = windowed.next();
assert.compareArray(result.value, [0, 1]);
assert.sameValue(result.done, false);

let { value, done } = iterator.next();
assert.sameValue(value, 2);
assert.sameValue(done, false);

result = windowed.next();
assert.compareArray(result.value, [1, 3]);
assert.sameValue(result.done, false);

result = windowed.next();
assert.compareArray(result.value, [3, 4]);
assert.sameValue(result.done, false);
