// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: search element is compared using SameValueZero
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  7. Repeat, while k < len
    a. Let elementK be the result of ? Get(O, ! ToString(k)).
    b. If SameValueZero(searchElement, elementK) is true, return true.
    c. Increase k by 1.
  ...
features: [Array.prototype.includes]
---*/

var sample = [42, 0, 1, NaN];
assert.sameValue(sample.includes("42"), false, "'42'");
assert.sameValue(sample.includes([42]), false, "[42]");
assert.sameValue(sample.includes(42.0), true, "42.0");
assert.sameValue(sample.includes(-0), true, "-0");
assert.sameValue(sample.includes(true), false, "true");
assert.sameValue(sample.includes(false), false, "false");
assert.sameValue(sample.includes(null), false, "null");
assert.sameValue(sample.includes(""), false, "empty string");
assert.sameValue(sample.includes(NaN), true, "NaN");
