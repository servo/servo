// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Test valid leap months when resolving fields in hebrew calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "hebrew";

// Valid leap month: Adar I 5779
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 5779, month: 6, day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }).toPlainDateTime(),
  5779, 6, "M05L", 1, 12, 34, 0, 0, 0, 0,
  "Leap month resolved from month number",
  "am", 5779);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 5779, monthCode: "M05L", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }).toPlainDateTime(),
  5779, 6, "M05L", 1, 12, 34, 0, 0, 0, 0,
  "Leap month resolved from month code",
  "am", 5779);

// Creating dates in later months in a leap year
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 5779, month: 7, day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }).toPlainDateTime(),
  5779, 7, "M06", 1, 12, 34, 0, 0, 0, 0,
  "Month after leap month resolved from month number",
  "am", 5779);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 5779, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }).toPlainDateTime(),
  5779, 7, "M06", 1, 12, 34, 0, 0, 0, 0,
  "Month after leap month resolved from month code",
  "am", 5779);
