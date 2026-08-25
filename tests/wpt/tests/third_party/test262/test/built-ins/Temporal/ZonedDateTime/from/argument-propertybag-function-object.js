// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Options may be a function object.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// "1976-11-18T00:00:00+01:00[+01:00]"
const expected = new Temporal.ZonedDateTime(217119600000000000n, "+01:00");

TemporalHelpers.assertZonedDateTimesEqual( Temporal.ZonedDateTime.from({
  year: 1976,
  month: 11,
  day: 18,
  timeZone: "+01:00"
}, () => {
}), expected);
