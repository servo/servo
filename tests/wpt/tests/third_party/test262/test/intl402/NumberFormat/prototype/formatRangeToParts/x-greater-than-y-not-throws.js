// Copyright 2022 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.NumberFormat.prototype.formatRangeToParts
description: >
  "formatRangeToParts" basic tests when argument  x > y, BigInt included and covers PartitionNumberRangePattern return a object.
info: |
  1.1.21 PartitionNumberRangePattern( numberFormat, x, y )
  (...)
features: [Intl.NumberFormat-v3]
---*/

const nf = new Intl.NumberFormat();

// If x > y, return a object.
assert.sameValue(typeof nf.formatRangeToParts(23, 12), "object",
    "should return object not throw RangeError");

// If x > y, return a object and both x and y are bigint.
assert.sameValue(typeof nf.formatRangeToParts(23n, 12n), "object",
    "should return object not throw RangeError");
//if y is -∞, return a object.
assert.sameValue(typeof nf.formatRangeToParts(23, -Infinity), "object",
    "should return object not throw RangeError");
//if y is -0 and x ≥ 0, return a object.
assert.sameValue(typeof nf.formatRangeToParts(23, -0), "object",
    "should return object not throw RangeError");
assert.sameValue(typeof nf.formatRangeToParts(0, -0), "object",
    "should return object not throw RangeError");

// if y is a mathematical value, return a object.
assert.sameValue(typeof nf.formatRangeToParts(Infinity, 23), "object",
    "should return object not throw RangeError");
// if y is -∞, return a object.
assert.sameValue(typeof nf.formatRangeToParts(Infinity, -Infinity), "object",
    "should return object not throw RangeError");
// if y is -0, return a object.
assert.sameValue(typeof nf.formatRangeToParts(Infinity, -0), "object",
    "should return object not throw RangeError");

// if y is a mathematical value and y < 0, return a object.
assert.sameValue(typeof nf.formatRangeToParts(-0, -1), "object",
    "should return object not throw RangeError");
// if y is -∞, return a object.
assert.sameValue(typeof nf.formatRangeToParts(-0, -Infinity), "object",
    "should return object not throw RangeError");
