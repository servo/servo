// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: >
  RangeError thrown when the value of roundingIncrement puts the ending bound
  out of range during the rounding operation
info: |
    NudgeToCalendarUnit ( sign, duration, destEpochNs, dateTime, calendarRec,
      timeZoneRec, increment, unit, roundingMode )

    6. Let _end_ be ? AddDateTime(_dateTime_.[[Year]], _dateTime_.[[Month]],
      _dateTime_.[[Day]], _dateTime_.[[Hour]], _dateTime_.[[Minute]],
      _dateTime_.[[Second]], _dateTime_.[[Millisecond]],
      _dateTime_.[[Microsecond]], _dateTime_.[[Nanosecond]], _calendarRec_,
      _endDuration_.[[Years]], _endDuration_.[[Months]],
      _endDuration_.[[Weeks]], _endDuration_.[[Days]],
      _endDuration_.[[NormalizedTime]], *undefined*).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.ZonedDateTime(0n, "UTC");
const later = new Temporal.ZonedDateTime(5n, "UTC");

assert.throws(
  RangeError,
  () => earlier.until(later, { smallestUnit: "days", roundingIncrement: 1e8 + 1 }),
  "ending bound of 1e8 + 1 days is out of range when added to 1970-01-01"
);
assert.throws(
  RangeError,
  () => later.until(earlier, { smallestUnit: "days", roundingIncrement: 1e8 + 1 }),
  "ending bound of -1e8 - 1 days is out of range when added to 1970-01-01"
);

{
  const result = earlier.until(later, { smallestUnit: "days", roundingIncrement: 1e8, roundingMode: "expand" });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 1e8, 0, 0, 0, 0, 0, 0,
    "ending bound of 1e8 days is not out of range when added to 1970-01-01");
}

{
  const result = later.until(earlier, { smallestUnit: "days", roundingIncrement: 1e8, roundingMode: "expand" });
  TemporalHelpers.assertDuration(result, 0, 0, 0, -1e8, 0, 0, 0, 0, 0, 0,
    "ending bound of -1e8 days is not out of range when added to 1970-01-01");
}
