// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: halfExpand value for roundingMode option
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 123, 987, 500);

const result1 = datetime.toString({ smallestUnit: "microsecond", roundingMode: "halfExpand" });
assert.sameValue(result1, "2000-05-02T12:34:56.123988",
  "roundingMode is halfExpand (with 6 digits from smallestUnit)");

const result2 = datetime.toString({ fractionalSecondDigits: 6, roundingMode: "halfExpand" });
assert.sameValue(result2, "2000-05-02T12:34:56.123988",
  "roundingMode is halfExpand (with 6 digits from fractionalSecondDigits)");

const result3 = datetime.toString({ smallestUnit: "millisecond", roundingMode: "halfExpand" });
assert.sameValue(result3, "2000-05-02T12:34:56.124",
  "roundingMode is halfExpand (with 3 digits from smallestUnit)");

const result4 = datetime.toString({ fractionalSecondDigits: 3, roundingMode: "halfExpand" });
assert.sameValue(result4, "2000-05-02T12:34:56.124",
  "roundingMode is halfExpand (with 3 digits from fractionalSecondDigits)");

const result5 = datetime.toString({ smallestUnit: "second", roundingMode: "halfExpand" });
assert.sameValue(result5, "2000-05-02T12:34:56",
  "roundingMode is halfExpand (with 0 digits from smallestUnit)");

const result6 = datetime.toString({ fractionalSecondDigits: 0, roundingMode: "halfExpand" });
assert.sameValue(result6, "2000-05-02T12:34:56",
  "roundingMode is halfExpand (with 0 digits from fractionalSecondDigits)");

const result7 = datetime.toString({ smallestUnit: "minute", roundingMode: "halfExpand" });
assert.sameValue(result7, "2000-05-02T12:35", "roundingMode is halfExpand (round to minute)");
