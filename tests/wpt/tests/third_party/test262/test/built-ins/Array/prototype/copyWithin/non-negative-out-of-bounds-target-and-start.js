// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Max values of target and start positions are this.length.
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  7. If relativeTarget < 0, let to be max((len + relativeTarget),0); else let to
  be min(relativeTarget, len).
  ...
  10. If relativeStart < 0, let from be max((len + relativeStart),0); else let
  from be min(relativeStart, len).
  11. If end is undefined, let relativeEnd be len; else let relativeEnd be
  ToInteger(end).
  ...
  14. Let count be min(final-from, len-to).
  15. If from<to and to<from+count
    ...
  16. Else,
    a. Let direction = 1.
  17. Repeat, while count > 0
    ...
  ...
includes: [compareArray.js]
---*/

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(6, 0), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(6, 0) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(7, 0), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(7, 0) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(Infinity, 0), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(Infinity, 0) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(6, 2), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(6, 2) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(7, 2), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(7, 2) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(Infinity, 2), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(Infinity, 2) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(0, 6), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(0, 6) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(0, 7), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(0, 7) must return [0, 1, 2, 3, 4, 5]'
);


assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(0, Infinity), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(0, Infinity) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(2, 6), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(2, 6) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(1, 7), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(1, 7) must return [0, 1, 2, 3, 4, 5]'
);


assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(3, Infinity), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(3, Infinity) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(6, 6), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(6, 6) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(10, 10), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(10, 10) must return [0, 1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(Infinity, Infinity), [0, 1, 2, 3, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(Infinity, Infinity) must return [0, 1, 2, 3, 4, 5]'
);
