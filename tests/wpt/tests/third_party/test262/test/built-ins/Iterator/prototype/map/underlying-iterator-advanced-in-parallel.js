// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.map
description: >
  Underlying iterator is advanced after calling map
info: |
  %Iterator.prototype%.map ( mapper )

features: [iterator-helpers]
flags: []
---*/
let iterator = (function* () {
  for (let i = 0; i < 5; ++i) {
    yield i;
  }
})();

let mapped = iterator.map(x => x);

let { value, done } = iterator.next();

assert.sameValue(value, 0);
assert.sameValue(done, false);

iterator.next();
iterator.next();

({ value, done } = mapped.next());

assert.sameValue(value, 3);
assert.sameValue(done, false);

({ value, done } = mapped.next());

assert.sameValue(value, 4);
assert.sameValue(done, false);

({ value, done } = mapped.next());

assert.sameValue(value, undefined);
assert.sameValue(done, true);
