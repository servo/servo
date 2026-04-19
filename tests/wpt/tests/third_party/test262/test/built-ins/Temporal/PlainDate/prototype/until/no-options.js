// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: With no options
info: |
  1. Let calendar be the this value.
  2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
  3. Assert: calendar.[[Identifier]] is "iso8601".
  4. Set one to ? ToTemporalDate(one).
  5. Set two to ? ToTemporalDate(two).
  6. Set options to ? GetOptionsObject(options).
  7. Let largestUnit be ? ToLargestTemporalUnit(options, « "hour", "minute", "second", "millisecond", "microsecond", "nanosecond" », "auto", "day").
  8. Let result be ! DifferenceISODate(one.[[ISOYear]], one.[[ISOMonth]], one.[[ISODay]], two.[[ISOYear]], two.[[ISOMonth]], two.[[ISODay]], largestUnit).
  9. Return ? CreateTemporalDuration(result.[[Years]], result.[[Months]], result.[[Weeks]], result.[[Days]], 0, 0, 0, 0, 0, 0).
features: [Temporal]
includes: [temporalHelpers.js]
---*/

TemporalHelpers.assertDuration(
      Temporal.PlainDate.from("2021-07-16").until("2021-07-16"),
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "same day");
TemporalHelpers.assertDuration(
      Temporal.PlainDate.from("2021-07-16").until("2021-07-17"),
      0, 0, 0, 1, 0, 0, 0, 0, 0, 0, "one day");
TemporalHelpers.assertDuration(
      Temporal.PlainDate.from("2021-07-16").until("2021-08-17"),
      0, 0, 0, 32, 0, 0, 0, 0, 0, 0, "32 days");
TemporalHelpers.assertDuration(
      Temporal.PlainDate.from("2021-07-16").until("2021-09-16"),
      0, 0, 0, 62, 0, 0, 0, 0, 0, 0, "62 days");
TemporalHelpers.assertDuration(
      Temporal.PlainDate.from("2021-07-16").until("2022-07-16"),
      0, 0, 0, 365, 0, 0, 0, 0, 0, 0, "365 days");
TemporalHelpers.assertDuration(
      Temporal.PlainDate.from("2021-07-16").until("2031-07-16"),
      0, 0, 0, 3652, 0, 0, 0, 0, 0, 0, "3652 days");

TemporalHelpers.assertDuration(
      Temporal.PlainDate.from("2021-07-17").until("2021-07-16"),
      0, 0, 0, -1, 0, 0, 0, 0, 0, 0, "negative one day");
TemporalHelpers.assertDuration(
      Temporal.PlainDate.from("2021-08-17").until("2021-07-16"),
      0, 0, 0, -32, 0, 0, 0, 0, 0, 0, "negative 32 days");
TemporalHelpers.assertDuration(
      Temporal.PlainDate.from("2021-09-16").until("2021-07-16"),
      0, 0, 0, -62, 0, 0, 0, 0, 0, 0, "negative 62 days");
TemporalHelpers.assertDuration(
      Temporal.PlainDate.from("2022-07-16").until("2021-07-16"),
      0, 0, 0, -365, 0, 0, 0, 0, 0, 0, "negative 365 days");
TemporalHelpers.assertDuration(
      Temporal.PlainDate.from("2031-07-16").until("2021-07-16"),
      0, 0, 0, -3652, 0, 0, 0, 0, 0, 0, "negative 3652 days");
