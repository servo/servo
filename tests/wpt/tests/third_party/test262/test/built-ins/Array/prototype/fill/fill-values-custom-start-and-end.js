// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.fill
description: >
  Fills all the elements from a with a custom start and end indexes.
info: |
  22.1.3.6 Array.prototype.fill (value [ , start [ , end ] ] )

  ...
  7. If relativeStart < 0, let k be max((len + relativeStart),0); else let k be
  min(relativeStart, len).
  8. If end is undefined, let relativeEnd be len; else let relativeEnd be
  ToInteger(end).
  9. ReturnIfAbrupt(relativeEnd).
  10. If relativeEnd < 0, let final be max((len + relativeEnd),0); else let
  final be min(relativeEnd, len).
  ...
includes: [compareArray.js]
---*/

assert.compareArray([0, 0, 0].fill(8, 1, 2), [0, 8, 0], '[0, 0, 0].fill(8, 1, 2) must return [0, 8, 0]');
assert.compareArray(
  [0, 0, 0, 0, 0].fill(8, -3, 4),
  [0, 0, 8, 8, 0],
  '[0, 0, 0, 0, 0].fill(8, -3, 4) must return [0, 0, 8, 8, 0]'
);
assert.compareArray(
  [0, 0, 0, 0, 0].fill(8, -2, -1),
  [0, 0, 0, 8, 0],
  '[0, 0, 0, 0, 0].fill(8, -2, -1) must return [0, 0, 0, 8, 0]'
);
assert.compareArray(
  [0, 0, 0, 0, 0].fill(8, -1, -3),
  [0, 0, 0, 0, 0],
  '[0, 0, 0, 0, 0].fill(8, -1, -3) must return [0, 0, 0, 0, 0]'
);
assert.compareArray([, , , , 0].fill(8, 1, 3), [, 8, 8, , 0], '[, , , , 0].fill(8, 1, 3) must return [, 8, 8, , 0]');
