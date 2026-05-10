// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.fill
description: >
  Fills all the elements with `value` from a defaul start and index.
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
  11. Repeat, while k < final
    a. Let Pk be ToString(k).
    b. Let setStatus be Set(O, Pk, value, true).
    c. ReturnIfAbrupt(setStatus).
    d. Increase k by 1.
  12. Return O.
includes: [compareArray.js]
---*/

assert.compareArray([].fill(8), [], '[].fill(8) must return []');

assert.compareArray([0, 0].fill(), [undefined, undefined], '[0, 0].fill() must return [undefined, undefined]');

assert.compareArray([0, 0, 0].fill(8), [8, 8, 8],
  '[0, 0, 0].fill(8) must return [8, 8, 8]'
);
