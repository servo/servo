// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: halfEven value for roundingMode option
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_123_987_500n);

const result1 = instant.toString({ smallestUnit: "microsecond", roundingMode: "halfEven" });
assert.sameValue(result1, "2001-09-09T01:46:40.123988Z",
  "roundingMode is halfEven (with 6 digits from smallestUnit)");

const result2 = instant.toString({ fractionalSecondDigits: 6, roundingMode: "halfEven" });
assert.sameValue(result2, "2001-09-09T01:46:40.123988Z",
  "roundingMode is halfEven (with 6 digits from fractionalSecondDigits)");

const result3 = instant.toString({ smallestUnit: "millisecond", roundingMode: "halfEven" });
assert.sameValue(result3, "2001-09-09T01:46:40.124Z",
  "roundingMode is halfEven (with 3 digits from smallestUnit)");

const result4 = instant.toString({ fractionalSecondDigits: 3, roundingMode: "halfEven" });
assert.sameValue(result4, "2001-09-09T01:46:40.124Z",
  "roundingMode is halfEven (with 3 digits from fractionalSecondDigits)");

const result5 = instant.toString({ smallestUnit: "second", roundingMode: "halfEven" });
assert.sameValue(result5, "2001-09-09T01:46:40Z",
  "roundingMode is halfEven (with 0 digits from smallestUnit)");

const result6 = instant.toString({ fractionalSecondDigits: 0, roundingMode: "halfEven" });
assert.sameValue(result6, "2001-09-09T01:46:40Z",
  "roundingMode is halfEven (with 0 digits from fractionalSecondDigits)");

const result7 = instant.toString({ smallestUnit: "minute", roundingMode: "halfEven" });
assert.sameValue(result7, "2001-09-09T01:47Z", "roundingMode is halfEven (round to minute)");
