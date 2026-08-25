// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Takes entries from this iterator until the limit is reached.
info: |
  %Iterator.prototype%.take ( limit )

  8.b.i If remaining is 0, then
    8.b.i.1. Return ? IteratorClose(iterated, NormalCompletion(undefined)).

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/
function* g() {
  let i = 0;
  while (true) {
    yield i;
    ++i;
  }
}

assert.compareArray(Array.from(g().take(0)), []);
assert.compareArray(Array.from(g().take(1)), [0]);
assert.compareArray(Array.from(g().take(2)), [0, 1]);
assert.compareArray(Array.from(g().take(3)), [0, 1, 2]);
