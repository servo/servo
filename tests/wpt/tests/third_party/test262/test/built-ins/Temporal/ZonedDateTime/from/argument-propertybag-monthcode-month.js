// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: ZonedDateTime can be constructed with monthCode or month; must agree.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// "1976-11-18T00:00:00+01:00[+01:00]"
const expected = new Temporal.ZonedDateTime(217119600000000000n, "+01:00");

// can be constructed with monthCode and without month
TemporalHelpers.assertZonedDateTimesEqual(Temporal.ZonedDateTime.from({
  year: 1976,
  monthCode: "M11",
  day: 18,
  timeZone: "+01:00"
}), expected);

// can be constructed with month and without monthCode
TemporalHelpers.assertZonedDateTimesEqual(Temporal.ZonedDateTime.from({
  year: 1976,
  month: 11,
  day: 18,
  timeZone: "+01:00"
}), expected)

// month and monthCode must agree
assert.throws(RangeError, () => Temporal.ZonedDateTime.from({
  year: 1976,
  month: 11,
  monthCode: "M12",
  day: 18,
  timeZone: "+01:00"
}));
