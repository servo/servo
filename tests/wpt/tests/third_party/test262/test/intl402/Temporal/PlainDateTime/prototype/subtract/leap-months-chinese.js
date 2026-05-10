// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Arithmetic around leap months in the chinese calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "chinese";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);

const leap193807L = Temporal.PlainDateTime.from({ year: 1938, monthCode: "M07L", day: 30, hour: 12, minute: 34, calendar }, options);
const leap195205L = Temporal.PlainDateTime.from({ year: 1952, monthCode: "M05L", day: 30, hour: 12, minute: 34, calendar }, options);
const leap196603L = Temporal.PlainDateTime.from({ year: 1966, monthCode: "M03L", day: 1, hour: 12, minute: 34, calendar }, options);
const common200008 = Temporal.PlainDateTime.from({ year: 2000, monthCode: "M08", day: 2, hour: 12, minute: 34, calendar }, options);
const common200108 = Temporal.PlainDateTime.from({ year: 2001, monthCode: "M08", day: 2, hour: 12, minute: 34, calendar }, options);
const common201901 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
const common201904 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M04", day: 1, hour: 12, minute: 34, calendar }, options);
const leap202004 = Temporal.PlainDateTime.from({ year: 2020, monthCode: "M04", day: 1, hour: 12, minute: 34, calendar }, options);
const leap202004L = Temporal.PlainDateTime.from({ year: 2020, monthCode: "M04L", day: 1, hour: 12, minute: 34, calendar }, options);
const common202104 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M04", day: 1, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  common201901.subtract(years1),
  2020, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "add 1 year from non-leap day"
);

TemporalHelpers.assertPlainDateTime(
  leap196603L.subtract(years1),
  1967, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "Adding 1 year to leap month M03L lands in common-year M03 with overflow constrain"
);

assert.throws(RangeError, function () {
  leap196603L.subtract(years1, options);
}, "Adding 1 year to leap month rejects");

TemporalHelpers.assertPlainDateTime(
  leap193807L.subtract(years1),
  1939, 7, "M07", 29, 12, 34, 0, 0, 0, 0, "Adding 1 year to leap month M07L on day 30 constrains to M07 day 29"
);

assert.throws(RangeError, function () {
  leap193807L.subtract(years1, options);
}, "Adding 1 year to leap month day 30 rejects");

TemporalHelpers.assertPlainDateTime(
  common201904.subtract(years1, options),
  2020, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Adding 1 year to common-year M04 lands in leap-year M04"
);

TemporalHelpers.assertPlainDateTime(
  leap202004.subtract(years1, options),
  2021, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Adding 1 year to leap-year M04 lands in common-year M04"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2012, monthCode: "M04L", day: 1, hour: 12, minute: 34, calendar }, options).subtract(new Temporal.Duration(-8), options),
  2020, 5, "M04L", 1, 12, 34, 0, 0, 0, 0, "Adding years to go from one M04L to the next M04L"
);

TemporalHelpers.assertPlainDateTime(
  common200008.subtract(years1, options),
  2001, 9, "M08", 2, 12, 34, 0, 0, 0, 0, "Adding 1 year crossing leap month"
);

TemporalHelpers.assertPlainDateTime(
  common201904.subtract(new Temporal.Duration(-2), options),
  2021, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Adding 2 years to common-year M04 crossing leap year lands in common-year M04"
);

TemporalHelpers.assertPlainDateTime(
  common201901.subtract(years1n),
  2018, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from non-leap day"
);

TemporalHelpers.assertPlainDateTime(
  leap196603L.subtract(years1n),
  1965, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from leap month M03L lands in common-year M03 with overflow constrain"
);

assert.throws(RangeError, function () {
  leap196603L.subtract(years1n, options);
}, "Subtracting 1 year from leap month rejects");

TemporalHelpers.assertPlainDateTime(
  leap195205L.subtract(years1n),
  1951, 5, "M05", 29, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from leap month M05L on day 30 constrains to M05 day 29"
);

assert.throws(RangeError, function () {
  leap195205L.subtract(years1n, options);
}, "Subtracting 1 year from leap month day 30 rejects");

TemporalHelpers.assertPlainDateTime(
  common202104.subtract(years1n, options),
  2020, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from common-year M04 lands in leap-year M04"
);

TemporalHelpers.assertPlainDateTime(
  leap202004.subtract(years1n, options),
  2019, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from leap-year M04 lands in common-year M04"
);

TemporalHelpers.assertPlainDateTime(
  leap202004L.subtract(new Temporal.Duration(8), options),
  2012, 5, "M04L", 1, 12, 34, 0, 0, 0, 0, "Subtracting years to go from one M04L to the previous M04L"
);

TemporalHelpers.assertPlainDateTime(
  common200108.subtract(years1n, options),
  2000, 8, "M08", 2, 12, 34, 0, 0, 0, 0, "Subtracting 1 year crossing leap month"
);

TemporalHelpers.assertPlainDateTime(
  common202104.subtract(new Temporal.Duration(2), options),
  2019, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Subtracting 2 years from common-year M04 crossing leap year lands in common-year M04"
);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months12 = new Temporal.Duration(0, -12);
const months12n = new Temporal.Duration(0, 12);
const months13 = new Temporal.Duration(0, -13);
const months13n = new Temporal.Duration(0, 13);

const leap202003 = Temporal.PlainDateTime.from({ year: 2020, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options);
const leap202006 = Temporal.PlainDateTime.from({ year: 2020, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1947, monthCode: "M02L", day: 1, hour: 12, minute: 34, calendar }, options).subtract(months1),
  1947, 4, "M03", 1, 12, 34, 0, 0, 0, 0, "add 1 month, starting at start of leap month"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1955, monthCode: "M03L", day: 1, hour: 12, minute: 34, calendar }, options).subtract(months1),
  1955, 5, "M04", 1, 12, 34, 0, 0, 0, 0, "add 1 month, starting at start of leap month with 30 days"
);

TemporalHelpers.assertPlainDateTime(
  leap202003.subtract(months1),
  2020, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "adding 1 month to M03 in leap year lands in M04 (not M04L)"
);

TemporalHelpers.assertPlainDateTime(
  leap202003.subtract(new Temporal.Duration(0, -2)),
  2020, 5, "M04L", 1, 12, 34, 0, 0, 0, 0, "adding 2 months to M03 in leap year lands in M04L (leap month)"
);

TemporalHelpers.assertPlainDateTime(
  leap202003.subtract(new Temporal.Duration(0, -3)),
  2020, 6, "M05", 1, 12, 34, 0, 0, 0, 0, "adding 3 months to M03 in leap year lands in M05 (not M06)"
);

TemporalHelpers.assertPlainDateTime(
  common201904.subtract(months12),
  2020, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Adding 12 months to common-year M04 lands in leap-year M04"
);

TemporalHelpers.assertPlainDateTime(
  common201904.subtract(months13),
  2020, 5, "M04L", 1, 12, 34, 0, 0, 0, 0, "Adding 13 months to common-year M04 lands in leap-year M04L"
);

TemporalHelpers.assertPlainDateTime(
  leap202004.subtract(months12),
  2021, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "Adding 12 months to leap-year M04 lands in common-year M03"
);

TemporalHelpers.assertPlainDateTime(
  leap202004.subtract(months13),
  2021, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Adding 13 months to leap-year M04 lands in common-year M04"
);

TemporalHelpers.assertPlainDateTime(
  leap202004L.subtract(months12),
  2021, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Adding 12 months to M04L lands in common-year M04"
);

TemporalHelpers.assertPlainDateTime(
  common200008.subtract(new Temporal.Duration(-1, -12), options),
  2002, 8, "M08", 2, 12, 34, 0, 0, 0, 0, "Adding 1y 12mo crossing leap month in the year part"
);

TemporalHelpers.assertPlainDateTime(
  common200108.subtract(new Temporal.Duration(-2, -13), options),
  2004, 9, "M08", 2, 12, 34, 0, 0, 0, 0, "Adding 1y 13mo crossing leap month in the months part"
);

TemporalHelpers.assertPlainDateTime(
  common201904.subtract(new Temporal.Duration(0, -24)),
  2021, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "Adding 24 months to common-year M04 crossing leap year with M04L, lands in common-year M03"
);

TemporalHelpers.assertPlainDateTime(
  common201904.subtract(new Temporal.Duration(0, -25)),
  2021, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Adding 25 months to common-year M04 crossing leap year with M04L, lands in common-year M04"
);

TemporalHelpers.assertPlainDateTime(
  leap202006.subtract(months1n),
  2020, 6, "M05", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 month from M06 in leap year lands in M05"
);

TemporalHelpers.assertPlainDateTime(
  leap202006.subtract(new Temporal.Duration(0, 2)),
  2020, 5, "M04L", 1, 12, 34, 0, 0, 0, 0, "Subtracting 2 months from M06 in leap year lands in M04L (leap month)"
);

TemporalHelpers.assertPlainDateTime(
  leap202006.subtract(new Temporal.Duration(0, 3)),
  2020, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Subtracting 3 months from M06 in leap year lands in M04 (not M03)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2020, monthCode: "M05", day: 1, hour: 12, minute: 34, calendar }, options).subtract(months1n),
  2020, 5, "M04L", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 month from M05 in leap year lands in M04L"
);

TemporalHelpers.assertPlainDateTime(
  leap202004L.subtract(months1n),
  2020, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 month from M04L in calendar lands in M04"
);

TemporalHelpers.assertPlainDateTime(
  common202104.subtract(months12n),
  2020, 5, "M04L", 1, 12, 34, 0, 0, 0, 0, "Subtracting 12 months from common-year M04 lands in leap-year M04L"
);

TemporalHelpers.assertPlainDateTime(
  common202104.subtract(months13n),
  2020, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Subtracting 13 months from common-year M04 lands in leap-year M04"
);

TemporalHelpers.assertPlainDateTime(
  leap202004.subtract(months12n),
  2019, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Subtracting 12 months from leap-year M04 lands in common-year M04"
);

TemporalHelpers.assertPlainDateTime(
  leap202004L.subtract(months12n),
  2019, 5, "M05", 1, 12, 34, 0, 0, 0, 0, "Subtracting 12 months from M04L lands in common-year M05"
);

TemporalHelpers.assertPlainDateTime(
  leap202004L.subtract(months13n),
  2019, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Subtracting 13 months from M04L lands in common-year M04"
);

TemporalHelpers.assertPlainDateTime(
  common200108.subtract(new Temporal.Duration(1, 12), options),
  1999, 8, "M08", 2, 12, 34, 0, 0, 0, 0, "Adding 1y 12mo crossing leap month in the year part"
);

TemporalHelpers.assertPlainDateTime(
  common200008.subtract(new Temporal.Duration(2, 13), options),
  1997, 8, "M08", 2, 12, 34, 0, 0, 0, 0, "Adding 1y 13mo crossing leap month in the months part"
);

TemporalHelpers.assertPlainDateTime(
  common202104.subtract(new Temporal.Duration(0, 24)),
  2019, 5, "M05", 1, 12, 34, 0, 0, 0, 0, "Subtracting 24 months from common-year M04 crossing leap year with M04L, lands in common-year M05"
);

TemporalHelpers.assertPlainDateTime(
  common202104.subtract(new Temporal.Duration(0, 25)),
  2019, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Subtracting 25 months from common-year M04 crossing leap year with M04L, lands in common-year M04"
);

// Weeks

const months2weeks3 = new Temporal.Duration(0, /* months = */ -2, /* weeks = */ -3);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1947, monthCode: "M02L", day: 29, hour: 12, minute: 34, calendar }, options).subtract(months2weeks3),
  1947, 6, "M05", 20, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks from last day leap month without leap day"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1955, monthCode: "M03L", day: 30, hour: 12, minute: 34, calendar }, options).subtract(months2weeks3),
  1955, 7, "M06", 21, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks from leap day in leap month"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1947, monthCode: "M01", day: 29, hour: 12, minute: 34, calendar }, options).subtract(months2weeks3),
  1947, 4, "M03", 21, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks from immediately before a leap month"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1955, monthCode: "M06", day: 29, hour: 12, minute: 34, calendar }, options).subtract(months2weeks3),
  1955, 10, "M09", 20, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks from immediately before a leap month"
);

// Days

const days10 = new Temporal.Duration(0, 0, 0, /* days = */ -10);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1955, monthCode: "M03L", day: 30, hour: 12, minute: 34, calendar }, options).subtract(days10),
  1955, 5, "M04", 10, 12, 34, 0, 0, 0, 0, "add 10 days from leap day in leap month"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1947, monthCode: "M02L", day: 29, hour: 12, minute: 34, calendar }, options).subtract(days10),
  1947, 4, "M03", 10, 12, 34, 0, 0, 0, 0, "add 10 days from last day of leap month"
);
