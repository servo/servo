// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  When windowSize is 1, each element is yielded as a single-element array
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
}

let windows = Array.from(g().windows(1));

assert.sameValue(windows.length, 5);
assert.compareArray(windows[0], [0]);
assert.compareArray(windows[1], [1]);
assert.compareArray(windows[2], [2]);
assert.compareArray(windows[3], [3]);
assert.compareArray(windows[4], [4]);
