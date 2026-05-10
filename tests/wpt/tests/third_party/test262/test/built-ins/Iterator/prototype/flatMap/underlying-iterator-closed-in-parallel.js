// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Underlying iterator is closed after calling flatMap
info: |
  %Iterator.prototype%.flatMap ( mapper )

features: [iterator-helpers]
flags: []
---*/
let iterator = (function* () {
  for (let i = 0; i < 5; ++i) {
    yield i;
  }
})();

let mapped = iterator.flatMap(x => [x]);

iterator.return();

let { value, done } = mapped.next();

assert.sameValue(value, undefined);
assert.sameValue(done, true);
