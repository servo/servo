// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: trunc value for roundingMode option
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_123_987_500n, "UTC");

const result1 = datetime.toString({ smallestUnit: "microsecond", roundingMode: "trunc" });
assert.sameValue(result1, "2001-09-09T01:46:40.123987+00:00[UTC]",
  "roundingMode is trunc (with 6 digits from smallestUnit)");

const result2 = datetime.toString({ fractionalSecondDigits: 6, roundingMode: "trunc" });
assert.sameValue(result2, "2001-09-09T01:46:40.123987+00:00[UTC]",
  "roundingMode is trunc (with 6 digits from fractionalSecondDigits)");

const result3 = datetime.toString({ smallestUnit: "millisecond", roundingMode: "trunc" });
assert.sameValue(result3, "2001-09-09T01:46:40.123+00:00[UTC]",
  "roundingMode is trunc (with 3 digits from smallestUnit)");

const result4 = datetime.toString({ fractionalSecondDigits: 3, roundingMode: "trunc" });
assert.sameValue(result4, "2001-09-09T01:46:40.123+00:00[UTC]",
  "roundingMode is trunc (with 3 digits from fractionalSecondDigits)");

const result5 = datetime.toString({ smallestUnit: "second", roundingMode: "trunc" });
assert.sameValue(result5, "2001-09-09T01:46:40+00:00[UTC]",
  "roundingMode is trunc (with 0 digits from smallestUnit)");

const result6 = datetime.toString({ fractionalSecondDigits: 0, roundingMode: "trunc" });
assert.sameValue(result6, "2001-09-09T01:46:40+00:00[UTC]",
  "roundingMode is trunc (with 0 digits from fractionalSecondDigits)");

const result7 = datetime.toString({ smallestUnit: "minute", roundingMode: "trunc" });
assert.sameValue(result7, "2001-09-09T01:46+00:00[UTC]", "roundingMode is trunc (round to minute)");
