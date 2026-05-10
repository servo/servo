// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Test behaviour around DST boundaries without any options set.
features: [Temporal]
---*/

const DSTEnd = {
  year: 2000,
  month: 10,
  day: 29,
  hour: 1,
  minute: 45,
  timeZone: "America/Vancouver"
};
assert.sameValue(
  Temporal.ZonedDateTime.from(DSTEnd).offset,
  "-07:00",
  "Ambiguous zoned date time");

const DSTStart = {
  year: 2000,
  month: 4,
  day: 2,
  hour: 2,
  minute: 30,
  timeZone: "America/Vancouver"
};
const zdt = Temporal.ZonedDateTime.from(DSTStart);
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result, zoned date time in non existent time");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result, zoned date time in non existent time");
