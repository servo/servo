// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Test valid leap months when resolving fields in hebrew calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "hebrew";

// Valid leap month: Adar I 5779
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 5779, month: 6, calendar }),
  5779, 6, "M05L",
  "Leap month resolved from month number",
  "am", 5779, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 5779, monthCode: "M05L", calendar }),
  5779, 6, "M05L",
  "Leap month resolved from month code",
  "am", 5779, null);

// Creating dates in later months in a leap year
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 5779, month: 7, calendar }),
  5779, 7, "M06",
  "Month after leap month resolved from month number",
  "am", 5779, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 5779, monthCode: "M06", calendar }),
  5779, 7, "M06",
  "Month after leap month resolved from month code",
  "am", 5779, null);
