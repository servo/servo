// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Iterator.prototype.windows with windowSize 3
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

features: [iterator-chunking, generators]
includes: [compareArray.js]
---*/
function* g() {
  yield 0;
  yield 1;
  yield 2;
  yield 3;
  yield 4;
  yield 5;
}

let windows = Array.from(g().windows(3));

assert.sameValue(windows.length, 4);
assert.compareArray(windows[0], [0, 1, 2]);
assert.compareArray(windows[1], [1, 2, 3]);
assert.compareArray(windows[2], [2, 3, 4]);
assert.compareArray(windows[3], [3, 4, 5]);
