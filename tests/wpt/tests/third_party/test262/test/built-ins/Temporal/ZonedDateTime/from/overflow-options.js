// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Overflow options.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const bad = {
  year: 2019,
  month: 1,
  day: 32,
  timeZone: "+01:00"
};
// "2019-01-31T00:00:00+01:00[+01:00]"
const expected = new Temporal.ZonedDateTime(1548889200000000000n, "+01:00");


assert.throws(RangeError, () => Temporal.ZonedDateTime.from(bad, { overflow: "reject" }));
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from(bad),
    expected);
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from(bad, { overflow: "constrain" }),
    expected);
