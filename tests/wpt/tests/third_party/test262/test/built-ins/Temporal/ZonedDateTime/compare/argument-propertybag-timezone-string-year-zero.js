// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Negative zero, as an extended year, is rejected
features: [Temporal, arrow-function]
---*/

const datetime = new Temporal.ZonedDateTime(0n, "UTC");
const invalidStrings = [
  "-000000-10-31T17:45Z",
  "-000000-10-31T17:45+00:00[UTC]",
];
invalidStrings.forEach((timeZone) => {
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.compare({ year: 2020, month: 5, day: 2, timeZone }, datetime),
    "reject minus zero as extended year (first argument)"
  );
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.compare(datetime, { year: 2020, month: 5, day: 2, timeZone }),
    "reject minus zero as extended year (second argument)"
  );
});
