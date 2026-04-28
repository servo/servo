// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Validation of offset
features: [Temporal]
---*/

// "+00:0000" is invalid (the hour/minute and minute/second separator
// or lack thereof needs to match). A valid offset would be either
// +00:00:00 or +000000.
["garbage", "00:00", "+000:00", "-00:000", "-00:00:000", "+00:00.0", "+00:00:00.0000000000", "+00:0000"].forEach((offset) => {
  assert.throws(RangeError, () => Temporal.ZonedDateTime.from({ offset, year: 2024, monthCode: "M10", day: 3, timeZone: "UTC" }),
    `UTC offset '${offset}' is not well-formed`);
});

assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.from({ offset: "--00:00", year: Symbol(), monthCode: "M10", day: 3, timeZone: "UTC" }),
  "UTC offset syntax is validated before year type is validated"
);

assert.throws(
  TypeError,
  () => Temporal.ZonedDateTime.from({ offset: "+04:30", year: Symbol(), monthCode: "M10", day: 3, timeZone: "UTC" }),
  "UTC offset matching is validated after year type is validated"
);
