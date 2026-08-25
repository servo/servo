// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: trunc value for roundingMode option
features: [Temporal]
---*/

const duration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 123, 987, 500);

const result1 = duration.toString({ smallestUnit: "microsecond", roundingMode: "trunc" });
assert.sameValue(result1, "P1Y2M3W4DT5H6M7.123987S",
  "roundingMode is trunc (with 6 digits from smallestUnit)");

const result2 = duration.toString({ fractionalSecondDigits: 6, roundingMode: "trunc" });
assert.sameValue(result2, "P1Y2M3W4DT5H6M7.123987S",
  "roundingMode is trunc (with 6 digits from fractionalSecondDigits)");

const result3 = duration.toString({ smallestUnit: "millisecond", roundingMode: "trunc" });
assert.sameValue(result3, "P1Y2M3W4DT5H6M7.123S",
  "roundingMode is trunc (with 3 digits from smallestUnit)");

const result4 = duration.toString({ fractionalSecondDigits: 3, roundingMode: "trunc" });
assert.sameValue(result4, "P1Y2M3W4DT5H6M7.123S",
  "roundingMode is trunc (with 3 digits from fractionalSecondDigits)");

const result5 = duration.toString({ smallestUnit: "second", roundingMode: "trunc" });
assert.sameValue(result5, "P1Y2M3W4DT5H6M7S",
  "roundingMode is trunc (with 0 digits from smallestUnit)");

const result6 = duration.toString({ fractionalSecondDigits: 0, roundingMode: "trunc" });
assert.sameValue(result6, "P1Y2M3W4DT5H6M7S",
  "roundingMode is trunc (with 0 digits from fractionalSecondDigits)");
