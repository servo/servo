// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Underlying iterator is closed after calling windows
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

features: [iterator-chunking, generators]
---*/
let iterator = (function* () {
  for (let i = 0; i < 5; ++i) {
    yield i;
  }
})();

let windowed = iterator.windows(2);

iterator.return();

let { value, done } = windowed.next();

assert.sameValue(value, undefined);
assert.sameValue(done, true);
