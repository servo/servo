// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.from
description: Constraining the day at end of month (hebrew calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

// Cheshvan and Kislev (02, 03) have 29 or 30 days, independent of leap years.
// Deficient - Cheshvan and Kislev have 29 days
// Regular - Cheshvan has 29 days, Kislev 30
// Complete - Cheshvan and Kislev have 30 days
//
// 5781 - a recent deficient year
// 5782 - a recent regular year

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 5781, monthCode: "M03", day: 30, calendar }),
  5781, 3, "M03", 29, "Kislev constrains to 29 in deficient year",
  "am", 5781);
assert.throws(RangeError, function () {
  Temporal.PlainDate.from({ year: 5781, monthCode: "M03", day: 30, calendar }, options);
}, "Kislev rejects 30 in deficient year");

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 5782, monthCode: "M02", day: 30, calendar }),
  5782, 2, "M02", 29, "Cheshvan constrains to 29 in regular year",
  "am", 5782);
assert.throws(RangeError, function () {
  Temporal.PlainDate.from({ year: 5782, monthCode: "M02", day: 30, calendar }, options);
}, "Cheshvan rejects 30 in regular year");

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 5781, monthCode: "M02", day: 30, calendar }),
  5781, 2, "M02", 29, "Cheshvan constrains to 29 in deficient year",
  "am", 5781);
assert.throws(RangeError, function () {
  Temporal.PlainDate.from({ year: 5781, monthCode: "M02", day: 30, calendar }, options);
}, "Cheshvan rejects 30 in deficient year");
