// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Takes entries from this iterator until it is exhausted or the limit is reached.
info: |
  %Iterator.prototype%.take ( limit )

  8.b.iii. Let next be ? IteratorStep(iterated).
  8.b.iv. If next is false, return undefined.

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 0;
  yield 1;
  yield 2;
}

assert.compareArray(Array.from(g().take(3)), [0, 1, 2]);
assert.compareArray(Array.from(g().take(4)), [0, 1, 2]);
assert.compareArray(Array.from(g().take(5)), [0, 1, 2]);
assert.compareArray(Array.from(g().take(Number.MAX_SAFE_INTEGER)), [0, 1, 2]);
assert.compareArray(Array.from(g().take(Infinity)), [0, 1, 2]);
