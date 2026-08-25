// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tostring
description: halfExpand value for roundingMode option
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 123, 987, 500);

const result1 = time.toString({ smallestUnit: "microsecond", roundingMode: "halfExpand" });
assert.sameValue(result1, "12:34:56.123988",
  "roundingMode is halfExpand (with 6 digits from smallestUnit)");

const result2 = time.toString({ fractionalSecondDigits: 6, roundingMode: "halfExpand" });
assert.sameValue(result2, "12:34:56.123988",
  "roundingMode is halfExpand (with 6 digits from fractionalSecondDigits)");

const result3 = time.toString({ smallestUnit: "millisecond", roundingMode: "halfExpand" });
assert.sameValue(result3, "12:34:56.124",
  "roundingMode is halfExpand (with 3 digits from smallestUnit)");

const result4 = time.toString({ fractionalSecondDigits: 3, roundingMode: "halfExpand" });
assert.sameValue(result4, "12:34:56.124",
  "roundingMode is halfExpand (with 3 digits from fractionalSecondDigits)");

const result5 = time.toString({ smallestUnit: "second", roundingMode: "halfExpand" });
assert.sameValue(result5, "12:34:56",
  "roundingMode is halfExpand (with 0 digits from smallestUnit)");

const result6 = time.toString({ fractionalSecondDigits: 0, roundingMode: "halfExpand" });
assert.sameValue(result6, "12:34:56",
  "roundingMode is halfExpand (with 0 digits from fractionalSecondDigits)");

const result7 = time.toString({ smallestUnit: "minute", roundingMode: "halfExpand" });
assert.sameValue(result7, "12:35", "roundingMode is halfExpand (round to minute)");
