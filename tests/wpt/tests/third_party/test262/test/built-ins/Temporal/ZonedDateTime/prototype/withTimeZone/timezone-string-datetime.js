// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withtimezone
description: Conversion of ISO date-time strings to time zone IDs
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "UTC");

let timeZone = "2021-08-19T17:30";
assert.throws(RangeError, () => instance.withTimeZone(timeZone), "bare date-time string is not a time zone");

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
    () => instance.withTimeZone(timeZone),
    `ISO string ${timeZone} with a sub-minute offset is not a valid time zone`
  );
});

timeZone = "2021-08-19T17:30Z";
const result1 = instance.withTimeZone(timeZone);
assert.sameValue(result1.timeZoneId, "UTC", "date-time + Z is UTC time zone");

timeZone = "2021-08-19T17:30-07:00";
const result2 = instance.withTimeZone(timeZone);
assert.sameValue(result2.timeZoneId, "-07:00", "date-time + offset is the offset time zone");

timeZone = "2021-08-19T17:30[UTC]";
const result3 = instance.withTimeZone(timeZone);
assert.sameValue(result3.timeZoneId, "UTC", "date-time + IANA annotation is the IANA time zone");

timeZone = "2021-08-19T17:30Z[UTC]";
const result4 = instance.withTimeZone(timeZone);
assert.sameValue(result4.timeZoneId, "UTC", "date-time + Z + IANA annotation is the IANA time zone");

timeZone = "2021-08-19T17:30-07:00[UTC]";
const result5 = instance.withTimeZone(timeZone);
assert.sameValue(result5.timeZoneId, "UTC", "date-time + offset + IANA annotation is the IANA time zone");
