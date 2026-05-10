// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Test behaviour around DST boundaries with the option offset set, when the argument is a property bag.
features: [Temporal]
---*/

// Ambiguous zoned date time - Fall DST
const DSTEnd = {
  year: 2000,
  month: 10,
  day: 29,
  timeZone: "America/Vancouver"
};

// First 1:30 when DST ends
let zdt = Temporal.ZonedDateTime.from({
  ...DSTEnd,
  hour: 1,
  minute: 30,
  offset: "-07:00"
}, { offset: "prefer" });
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option offset: prefer and bag's offset matches time zone, ambiguous time (first 1:30)");

// Second 1:30 when DST ends
zdt = Temporal.ZonedDateTime.from({
  ...DSTEnd,
  hour: 1,
  minute: 30,
  offset: "-08:00"
}, { offset: "prefer" });
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when option offset: prefer and bag's offset matches time zone, ambiguous time (second 1:30)");

zdt = Temporal.ZonedDateTime.from({
  ...DSTEnd,
  hour: 4,
  offset: "-07:00"
}, { offset: "prefer" });
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when option offset: prefer, and bag's offset does not match time zone, ambiguous time");
assert.sameValue(
  zdt.hour,
  4,
  "Hour result when option offset: prefer, and bag's offset does not match time zone, ambiguous time");

zdt = Temporal.ZonedDateTime.from({
  ...DSTEnd,
  hour: 4,
  offset: "-12:00"
}, { offset: "ignore" });
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when option offset: ignore, and bag's offset does not match time zone, ambiguous time");
assert.sameValue(
  zdt.hour,
  4,
  "Hour result when option offset: ignore, and bag's offset does not match time zone, ambiguous time");

// The option { offset: 'use' } does not use a wrong offset.
zdt = Temporal.ZonedDateTime.from({
  ...DSTEnd,
  hour: 4,
  offset: "-07:00"
}, { offset: "use" });
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when option offset: use, and bag's offset is wrong, ambiguous time");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when option offset: use, and bag's offset is wrong, ambiguous time");

// Non existent zoned date time - Spring DST
const DSTStart = {
  year: 2000,
  month: 4,
  day: 2,
  hour: 2,
  minute: 30,
  timeZone: "America/Vancouver"
};

zdt = Temporal.ZonedDateTime.from(
  DSTStart,
  { offset: "ignore" });
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option offset: ignore, non existent time");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when option offset: ignore, non existent time");

zdt = Temporal.ZonedDateTime.from({
  ...DSTStart,
  offset: "-23:59"
}, { offset: "prefer" });
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when offset is wrong and option offset: prefer, non existent time");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when offset is wrong and option offset: prefer, non existent time");
