// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  target argument is coerced to an integer value.
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  5. Let relativeTarget be ToInteger(target).
  ...
includes: [compareArray.js]
---*/

assert.compareArray(
  [0, 1, 2, 3].copyWithin(undefined, 1), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(undefined, 1) must return [1, 2, 3, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(false, 1), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(false, 1) must return [1, 2, 3, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(NaN, 1), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(NaN, 1) must return [1, 2, 3, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(null, 1), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(null, 1) must return [1, 2, 3, 3]'
);


assert.compareArray(
  [0, 1, 2, 3].copyWithin(true, 0), [0, 0, 1, 2],
  '[0, 1, 2, 3].copyWithin(true, 0) must return [0, 0, 1, 2]'
);


assert.compareArray(
  [0, 1, 2, 3].copyWithin('1', 0), [0, 0, 1, 2],
  '[0, 1, 2, 3].copyWithin("1", 0) must return [0, 0, 1, 2]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(0.5, 1), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(0.5, 1) must return [1, 2, 3, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1.5, 0), [0, 0, 1, 2],
  '[0, 1, 2, 3].copyWithin(1.5, 0) must return [0, 0, 1, 2]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin({}, 1), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin({}, 1) must return [1, 2, 3, 3]'
);
