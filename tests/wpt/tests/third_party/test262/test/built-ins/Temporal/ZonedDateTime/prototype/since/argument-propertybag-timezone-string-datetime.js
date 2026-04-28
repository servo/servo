// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Conversion of ISO date-time strings to Temporal.TimeZone instances
features: [Temporal]
---*/

let expectedTimeZone = "UTC";
const instance1 = new Temporal.ZonedDateTime(0n, expectedTimeZone);

let timeZone = "2021-08-19T17:30";
assert.throws(RangeError, () => instance1.since({ year: 2020, month: 5, day: 2, timeZone }), "bare date-time string is not a time zone");

[
  "2021-08-19T17:30-07:00:01",
  "2021-08-19T17:30-07:00:00",
  "2021-08-19T17:30-07:00:00.1",
  "2021-08-19T17:30-07:00:00.0",
  "2021-08-19T17:30-07:00:00.01",
  "2021-08-19T17:30-07:00:00.00",
  "2021-08-19T17:30-07:00:00.001",
  "2021-08-19T17:30-07:00:00.000",
  "2021-08-19T17:30-07:00:00.0001",
  "2021-08-19T17:30-07:00:00.0000",
  "2021-08-19T17:30-07:00:00.00001",
  "2021-08-19T17:30-07:00:00.00000",
  "2021-08-19T17:30-07:00:00.000001",
  "2021-08-19T17:30-07:00:00.000000",
  "2021-08-19T17:30-07:00:00.0000001",
  "2021-08-19T17:30-07:00:00.0000000",
  "2021-08-19T17:30-07:00:00.00000001",
  "2021-08-19T17:30-07:00:00.00000000",
  "2021-08-19T17:30-07:00:00.000000001",
  "2021-08-19T17:30-07:00:00.000000000",
].forEach((timeZone) => {
  assert.throws(
    RangeError,
    () => instance1.since({ year: 2020, month: 5, day: 2, timeZone }),
    `ISO string ${timeZone} with a sub-minute offset is not a valid time zone`
  );
});

// The following are all valid strings so should not throw. They should produce
// expectedTimeZone, so additionally the operation will not throw due to the
// time zones being different on the receiver and the argument.

timeZone = "2021-08-19T17:30Z";
instance1.since({ year: 2020, month: 5, day: 2, timeZone });

expectedTimeZone = "-07:00";
const instance2 = new Temporal.ZonedDateTime(0n, expectedTimeZone);
timeZone = "2021-08-19T17:30-07:00";
instance2.since({ year: 2020, month: 5, day: 2, timeZone });

expectedTimeZone = "UTC";
const instance3 = new Temporal.ZonedDateTime(0n, expectedTimeZone);
timeZone = "2021-08-19T17:30[UTC]";
instance3.since({ year: 2020, month: 5, day: 2, timeZone });

timeZone = "2021-08-19T17:30Z[UTC]";
instance3.since({ year: 2020, month: 5, day: 2, timeZone });

timeZone = "2021-08-19T17:30-07:00[UTC]";
instance3.since({ year: 2020, month: 5, day: 2, timeZone });
