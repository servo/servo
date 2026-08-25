// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: Searches all indexes from a sparse array
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  5. If n ≥ 0, then
    a. Let k be n.
  6. Else n < 0,
    a. Let k be len + n.
    b. If k < 0, let k be 0.
  7. Repeat, while k < len
    a. Let elementK be the result of ? Get(O, ! ToString(k)).
    b. If SameValueZero(searchElement, elementK) is true, return true.
    c. Increase k by 1.
  ...
features: [Array.prototype.includes]
---*/

assert.sameValue(
  [, , , ].includes(undefined),
  true,
  "[ , , , ].includes(undefined)"
);

assert.sameValue(
  [, , , 42, ].includes(undefined, 4),
  false,
  "[ , , , 42, ].includes(undefined, 4)"
);

var sample = [, , , 42, , ];

assert.sameValue(
  sample.includes(undefined),
  true,
  "sample.includes(undefined)"
);
assert.sameValue(
  sample.includes(undefined, 4),
  true,
  "sample.includes(undefined, 4)"
);
assert.sameValue(
  sample.includes(42, 3),
  true,
  "sample.includes(42, 3)"
);
