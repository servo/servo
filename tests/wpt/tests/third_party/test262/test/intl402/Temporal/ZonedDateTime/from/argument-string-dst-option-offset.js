// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Test behaviour around DST boundaries with the option offset set, when the argument is a string.
features: [Temporal]
---*/

// Fall DST end
let zdt = Temporal.ZonedDateTime.from(
  "2020-11-01T01:30-07:00[America/Los_Angeles]",
  { offset: "prefer" });
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option offset: prefer and offset matches time zone (first 1:30 when DST ends)");

zdt = Temporal.ZonedDateTime.from(
  "2020-11-01T01:30[America/Los_Angeles]",
  { offset: "prefer" });
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option offset: prefer and string argument is ambiguous");

zdt = Temporal.ZonedDateTime.from(
  "2020-11-01T01:30-08:00[America/Los_Angeles]",
  { offset: "prefer" });
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when option offset: prefer and offset matches time zone (second 1:30 when DST ends)");

zdt = Temporal.ZonedDateTime.from(
  "2020-11-01T04:00-07:00[America/Los_Angeles]",
  { offset: "prefer" });
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when option offset: prefer and offset does not match time zone, DST end");
assert.sameValue(
  zdt.hour,
  4,
  "Hour result when option offset: prefer and offset does not match time zone, DST end");

zdt = Temporal.ZonedDateTime.from(
  "2020-11-01T04:00-07:00[America/Los_Angeles]",
  { offset: "use" });
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when option offset: use and wrong offset, DST end");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when option offset: use and wrong offset, DST end");

zdt = Temporal.ZonedDateTime.from(
  "2020-11-01T04:00-12:00[America/Los_Angeles]",
  { offset: "ignore" });
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when option offset: ignore and wrong offset, DST end");
assert.sameValue(
  zdt.hour,
  4,
  "Hour result when option offset: ignore and wrong offset, DST end");

// Spring DST start
zdt = Temporal.ZonedDateTime.from(
  "2020-03-08T02:30[America/Los_Angeles]",
  { offset: "ignore" });
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option offset: ignore, DST start");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when option offset: ignore, DST start");

zdt = Temporal.ZonedDateTime.from(
  "2020-03-08T02:30-23:59[America/Los_Angeles]",
  { offset: "prefer" });
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option offset: ignore and wrong offset, DST start");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when option offset: ignore and wrong offset, DST start");
