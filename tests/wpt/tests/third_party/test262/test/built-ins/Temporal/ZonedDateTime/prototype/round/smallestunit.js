// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: Test various smallestUnit values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// const bal = Temporal.ZonedDateTime.from("1976-11-18T23:59:59.999999999+01:00[+01:00]");
const bal = new Temporal.ZonedDateTime(217205999999999999n, "+01:00");
// "1976-11-19T00:00:00+01:00[+01:00]"
const expected = new Temporal.ZonedDateTime(217206000000000000n, "+01:00");

[
  "day",
  "hour",
  "minute",
  "second",
  "millisecond",
  "microsecond"
].forEach(smallestUnit => {
    TemporalHelpers.assertZonedDateTimesEqual(bal.round( { smallestUnit }),
                                              expected);
});
