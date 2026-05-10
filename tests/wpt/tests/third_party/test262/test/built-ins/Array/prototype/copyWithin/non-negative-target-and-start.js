// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Copy values with non-negative target and start positions.
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
    a. If fromPresent is true, then
      i. Let fromVal be Get(O, fromKey).
      ...
      iii. Let setStatus be Set(O, toKey, fromVal, true).
  ...
includes: [compareArray.js]
---*/

assert.compareArray(
  ['a', 'b', 'c', 'd', 'e', 'f'].copyWithin(0, 0),
  ['a', 'b', 'c', 'd', 'e', 'f']
);

assert.compareArray(
  ['a', 'b', 'c', 'd', 'e', 'f'].copyWithin(0, 2),
  ['c', 'd', 'e', 'f', 'e', 'f']
);

assert.compareArray(
  ['a', 'b', 'c', 'd', 'e', 'f'].copyWithin(3, 0),
  ['a', 'b', 'c', 'a', 'b', 'c']
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].copyWithin(1, 4),
  [0, 4, 5, 3, 4, 5]
);
