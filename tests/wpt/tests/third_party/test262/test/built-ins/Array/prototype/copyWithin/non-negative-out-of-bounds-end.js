// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Max value of end position is the this.length.
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
    a. Let direction be -1.
    b. Let from be from + count -1.
    c. Let to be to + count -1.
  16. Else,
    a. Let direction = 1.
  17. Repeat, while count > 0
    ...
    a. If fromPresent is true, then
      i. Let fromVal be Get(O, fromKey).
      ...
      iii. Let setStatus be Set(O, toKey, fromVal, true).
  ...
includes: [compareArray.js]
---*/

assert.compareArray(
  [0, 1, 2, 3].copyWithin(0, 1, 6), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(0, 1, 6) must return [1, 2, 3, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(0, 1, Infinity), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(0, 1, Infinity) must return [1, 2, 3, 3]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(1, 3, 6), [0, 3, 4, 5, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(1, 3, 6) must return [0, 3, 4, 5, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(1, 3, Infinity), [0, 3, 4, 5, 4, 5],
  '[0, 1, 2, 3, 4, 5].copyWithin(1, 3, Infinity) must return [0, 3, 4, 5, 4, 5]'
);
