// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Iterator.prototype.windows yields sliding windows of the specified size
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  8.a.iii. If the number of elements in buffer is ℝ(windowSize), then
    8.a.iii.a. Remove the first element from buffer.
  8.a.iv. Append value to buffer.
  8.a.v. If the number of elements in buffer is ℝ(windowSize), then
    8.a.v.a. Let completion be Completion(Yield(CreateArrayFromList(buffer))).

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

let windows = Array.from(g().windows(2));

assert.sameValue(windows.length, 4);
assert.compareArray(windows[0], [0, 1]);
assert.compareArray(windows[1], [1, 2]);
assert.compareArray(windows[2], [2, 3]);
assert.compareArray(windows[3], [3, 4]);
