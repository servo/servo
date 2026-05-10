// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  start argument is coerced to an integer value.
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  8. Let relativeStart be ToInteger(start).
  ...
includes: [compareArray.js]
---*/

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1, undefined), [0, 0, 1, 2],
  '[0, 1, 2, 3].copyWithin(1, undefined) must return [0, 0, 1, 2]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1, false), [0, 0, 1, 2],
  '[0, 1, 2, 3].copyWithin(1, false) must return [0, 0, 1, 2]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1, NaN), [0, 0, 1, 2],
  '[0, 1, 2, 3].copyWithin(1, NaN) must return [0, 0, 1, 2]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1, null), [0, 0, 1, 2],
  '[0, 1, 2, 3].copyWithin(1, null) must return [0, 0, 1, 2]'
);


assert.compareArray(
  [0, 1, 2, 3].copyWithin(0, true), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(0, true) must return [1, 2, 3, 3]'
);


assert.compareArray(
  [0, 1, 2, 3].copyWithin(0, '1'), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(0, "1") must return [1, 2, 3, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1, 0.5), [0, 0, 1, 2],
  '[0, 1, 2, 3].copyWithin(1, 0.5) must return [0, 0, 1, 2]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(0, 1.5), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(0, 1.5) must return [1, 2, 3, 3]'
);
