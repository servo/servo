// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Underlying iterator is advanced after calling take
info: |
  %Iterator.prototype%.take ( limit )

features: [iterator-helpers]
flags: []
---*/
let iterator = (function* () {
  for (let i = 0; i < 5; ++i) {
    yield i;
  }
})();

let taken = iterator.take(2);

let { value, done } = iterator.next();

assert.sameValue(value, 0);
assert.sameValue(done, false);

({ value, done } = taken.next());

assert.sameValue(value, 1);
assert.sameValue(done, false);

({ value, done } = taken.next());

assert.sameValue(value, 2);
assert.sameValue(done, false);

({ value, done } = taken.next());

assert.sameValue(value, undefined);
assert.sameValue(done, true);
