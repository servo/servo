// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Constraining the day for 29/30-day months in islamic-civil calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-civil";
const options = { overflow: "reject" };

// 30-day months: 01, 03, 05, 07, 09, 11
// 29-day months: 02, 04, 06, 08, 10
// Month 12 (Dhu al-Hijjah) has 29 days in common years and 30 in leap years.
// 1445 is a leap year, 1444 a common year.

const common0130 = Temporal.PlainDateTime.from({ year: 1444, monthCode: "M01", day: 30, hour: 12, minute: 34, calendar }, options);
const leap0130 = Temporal.PlainDateTime.from({ year: 1445, monthCode: "M01", day: 30, hour: 12, minute: 34, calendar }, options);

// Common year

TemporalHelpers.assertPlainDateTime(
  common0130.with({ monthCode: "M02" }),
  1444, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "common-year Safar constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  common0130.with({ monthCode: "M02" }, options);
}, "common-year Safar rejects with 30");

TemporalHelpers.assertPlainDateTime(
  common0130.with({ monthCode: "M03" }, options),
  1444, 3, "M03", 30, 12, 34, 0, 0, 0, 0, "common-year Rabi' al-Awwal does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDateTime(
  common0130.with({ monthCode: "M04" }),
  1444, 4, "M04", 29, 12, 34, 0, 0, 0, 0, "common-year Rabi' al-Thani constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  common0130.with({ monthCode: "M04" }, options);
}, "common-year Rabi' al-Thani rejects with 30");

TemporalHelpers.assertPlainDateTime(
  common0130.with({ monthCode: "M05" }, options),
  1444, 5, "M05", 30, 12, 34, 0, 0, 0, 0, "common-year Jumada al-Awwal does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDateTime(
  common0130.with({ monthCode: "M06" }),
  1444, 6, "M06", 29, 12, 34, 0, 0, 0, 0, "common-year Jumada al-Thani constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  common0130.with({ monthCode: "M06" }, options);
}, "common-year Jumada al-Thani rejects with 30");

TemporalHelpers.assertPlainDateTime(
  common0130.with({ monthCode: "M07" }, options),
  1444, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "common-year Rajab does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDateTime(
  common0130.with({ monthCode: "M08" }),
  1444, 8, "M08", 29, 12, 34, 0, 0, 0, 0, "common-year Sha'ban constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  common0130.with({ monthCode: "M08" }, options);
}, "common-year Sha'ban rejects with 30");

TemporalHelpers.assertPlainDateTime(
  common0130.with({ monthCode: "M09" }, options),
  1444, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "common-year Ramadan does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDateTime(
  common0130.with({ monthCode: "M10" }),
  1444, 10, "M10", 29, 12, 34, 0, 0, 0, 0, "common-year Shawwal constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  common0130.with({ monthCode: "M10" }, options);
}, "common-year Shawwal rejects with 30");

TemporalHelpers.assertPlainDateTime(
  common0130.with({ monthCode: "M11" }, options),
  1444, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "common-year Dhu al-Qadah does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDateTime(
  common0130.with({ monthCode: "M12" }),
  1444, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "common-year Dhu al-Hijjah constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  common0130.with({ monthCode: "M12" }, options);
}, "common-year Dhu al-Hijjah rejects with 30");

// Leap year

TemporalHelpers.assertPlainDateTime(
  leap0130.with({ monthCode: "M02" }),
  1445, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "leap-year Safar constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  leap0130.with({ monthCode: "M02" }, options);
}, "leap-year Safar rejects with 30");

TemporalHelpers.assertPlainDateTime(
  leap0130.with({ monthCode: "M03" }, options),
  1445, 3, "M03", 30, 12, 34, 0, 0, 0, 0, "leap-year Rabi' al-Awwal does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDateTime(
  leap0130.with({ monthCode: "M04" }),
  1445, 4, "M04", 29, 12, 34, 0, 0, 0, 0, "leap-year Rabi' al-Thani constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  leap0130.with({ monthCode: "M04" }, options);
}, "leap-year Rabi' al-Thani rejects with 30");

TemporalHelpers.assertPlainDateTime(
  leap0130.with({ monthCode: "M05" }, options),
  1445, 5, "M05", 30, 12, 34, 0, 0, 0, 0, "leap-year Jumada al-Awwal does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDateTime(
  leap0130.with({ monthCode: "M06" }),
  1445, 6, "M06", 29, 12, 34, 0, 0, 0, 0, "leap-year Jumada al-Thani constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  leap0130.with({ monthCode: "M06" }, options);
}, "leap-year Jumada al-Thani rejects with 30");

TemporalHelpers.assertPlainDateTime(
  leap0130.with({ monthCode: "M07" }, options),
  1445, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "leap-year Rajab does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDateTime(
  leap0130.with({ monthCode: "M08" }),
  1445, 8, "M08", 29, 12, 34, 0, 0, 0, 0, "leap-year Sha'ban constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  leap0130.with({ monthCode: "M08" }, options);
}, "leap-year Sha'ban rejects with 30");

TemporalHelpers.assertPlainDateTime(
  leap0130.with({ monthCode: "M09" }, options),
  1445, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "leap-year Ramadan does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDateTime(
  leap0130.with({ monthCode: "M10" }),
  1445, 10, "M10", 29, 12, 34, 0, 0, 0, 0, "leap-year Shawwal constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  leap0130.with({ monthCode: "M10" }, options);
}, "leap-year Shawwal rejects with 30");

TemporalHelpers.assertPlainDateTime(
  leap0130.with({ monthCode: "M11" }, options),
  1445, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "leap-year Dhu al-Qadah does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDateTime(
  leap0130.with({ monthCode: "M12" }, options),
  1445, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "leap-year Dhu al-Hijjah does not reject 30",
  "ah", 1445);
