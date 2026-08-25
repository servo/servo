// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Each yielded window is a distinct new Array object
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  8.a.v.a. Let completion be Completion(Yield(CreateArrayFromList(buffer))).

features: [iterator-chunking, generators]
---*/
function* g() {
  yield 0;
  yield 1;
  yield 2;
  yield 3;
}

let windows = Array.from(g().windows(2));

assert.sameValue(windows.length, 3);
assert(Array.isArray(windows[0]), 'windows[0] is an Array');
assert(Array.isArray(windows[1]), 'windows[1] is an Array');
assert(Array.isArray(windows[2]), 'windows[2] is an Array');
assert.notSameValue(windows[0], windows[1], 'windows[0] !== windows[1]');
assert.notSameValue(windows[1], windows[2], 'windows[1] !== windows[2]');
assert.notSameValue(windows[0], windows[2], 'windows[0] !== windows[2]');
