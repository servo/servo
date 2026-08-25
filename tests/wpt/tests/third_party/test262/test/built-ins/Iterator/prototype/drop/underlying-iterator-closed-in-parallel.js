// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  Underlying iterator is closed after calling drop
info: |
  %Iterator.prototype%.drop ( limit )

features: [iterator-helpers]
flags: []
---*/
let iterator = (function* () {
  for (let i = 0; i < 5; ++i) {
    yield i;
  }
})();

let dropped = iterator.drop(2);

iterator.return();

let { value, done } = dropped.next();

assert.sameValue(value, undefined);
assert.sameValue(done, true);
