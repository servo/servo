// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.NumberFormat-formatRangeToParts
description: >
  "formatRangeToParts" Throws a RangeError if some of arguments is cast to NaN
info: |
  Intl.NumberFormat.prototype.formatRangeToParts( start, end )
  (...)
  6. Return ? FormatNumericRangeToParts(nf, x, y).

  FormatNumericRangeToParts( numberFormat, x, y )
  1. Let parts be ? PartitionNumberRangePattern(numberFormat, x, y).

  PartitionNumberRangePattern( numberFormat, x, y )
  1. If x is NaN or y is NaN, throw a RangeError exception.
features: [Intl.NumberFormat-v3]
---*/


const nf = new Intl.NumberFormat();

// If x or y is NaN ..., throw a RangeError exception.
assert.throws(RangeError, () => { nf.formatRangeToParts(NaN, 23) });
assert.throws(RangeError, () => { nf.formatRangeToParts(12, NaN) });
assert.throws(RangeError, () => { nf.formatRangeToParts(NaN, -23) });
assert.throws(RangeError, () => { nf.formatRangeToParts(-12, NaN) });
assert.throws(RangeError, () => { nf.formatRangeToParts(NaN, NaN) });
