// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  When undersized is "allow-partial", a partial final window is yielded if
  the buffer is non-empty and smaller than windowSize at exhaustion
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  8.a.ii. If value is ~done~, then
    8.a.ii.a. If undersized is "allow-partial", buffer is not empty, and the number of elements in buffer < ℝ(windowSize), then
      8.a.ii.a.i. Perform Completion(Yield(CreateArrayFromList(buffer))).
    8.a.ii.b. Return ReturnCompletion(undefined).

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

let windows = Array.from(g().windows(100, 'allow-partial'));

assert.sameValue(windows.length, 1);
assert.compareArray(windows[0], [0, 1, 2, 3, 4, 5]);
