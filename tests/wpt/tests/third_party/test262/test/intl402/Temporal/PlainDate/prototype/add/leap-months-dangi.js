// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Arithmetic around leap months in the dangi calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "dangi";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);

const leap193807L = Temporal.PlainDate.from({ year: 1938, monthCode: "M07L", day: 30, calendar }, options);
const leap195205L = Temporal.PlainDate.from({ year: 1952, monthCode: "M05L", day: 30, calendar }, options);
const leap196603L = Temporal.PlainDate.from({ year: 1966, monthCode: "M03L", day: 1, calendar }, options);
const common200008 = Temporal.PlainDate.from({ year: 2000, monthCode: "M08", day: 2, calendar }, options);
const common200108 = Temporal.PlainDate.from({ year: 2001, monthCode: "M08", day: 2, calendar }, options);
const common201901 = Temporal.PlainDate.from({ year: 2019, monthCode: "M01", day: 1, calendar }, options);
const common201904 = Temporal.PlainDate.from({ year: 2019, monthCode: "M04", day: 1, calendar }, options);
const leap202004 = Temporal.PlainDate.from({ year: 2020, monthCode: "M04", day: 1, calendar }, options);
const leap202004L = Temporal.PlainDate.from({ year: 2020, monthCode: "M04L", day: 1, calendar }, options);
const common202104 = Temporal.PlainDate.from({ year: 2021, monthCode: "M04", day: 1, calendar }, options);

TemporalHelpers.assertPlainDate(
  common201901.add(years1),
  2020, 1, "M01", 1, "add 1 year from non-leap day"
);

TemporalHelpers.assertPlainDate(
  leap196603L.add(years1),
  1967, 3, "M03", 1, "Adding 1 year to leap month M03L lands in common-year M03 with overflow constrain"
);

assert.throws(RangeError, function () {
  leap196603L.add(years1, options);
}, "Adding 1 year to leap month rejects");

TemporalHelpers.assertPlainDate(
  leap193807L.add(years1),
  1939, 7, "M07", 29, "Adding 1 year to leap month M07L on day 30 constrains to M07 day 29"
);

assert.throws(RangeError, function () {
  leap193807L.add(years1, options);
}, "Adding 1 year to leap month day 30 rejects");

TemporalHelpers.assertPlainDate(
  common201904.add(years1, options),
  2020, 4, "M04", 1, "Adding 1 year to common-year M04 lands in leap-year M04"
);

TemporalHelpers.assertPlainDate(
  leap202004.add(years1, options),
  2021, 4, "M04", 1, "Adding 1 year to leap-year M04 lands in common-year M04"
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2012, monthCode: "M03L", day: 1, calendar }, options).add(new Temporal.Duration(-19), options),
  1993, 4, "M03L", 1, "Subtracting years to go from one M03L to the previous M03L"
);

TemporalHelpers.assertPlainDate(
  common200008.add(years1, options),
  2001, 9, "M08", 2, "Adding 1 year crossing leap month"
);

TemporalHelpers.assertPlainDate(
  common201904.add(new Temporal.Duration(2), options),
  2021, 4, "M04", 1, "Adding 2 years to common-year M04 crossing leap year lands in common-year M04"
);

TemporalHelpers.assertPlainDate(
  common201901.add(years1n),
  2018, 1, "M01", 1, "Subtracting 1 year from non-leap day"
);

TemporalHelpers.assertPlainDate(
  leap196603L.add(years1n),
  1965, 3, "M03", 1, "Subtracting 1 year from leap month M03L lands in common-year M03 with overflow constrain"
);

assert.throws(RangeError, function () {
  leap196603L.add(years1n, options);
}, "Subtracting 1 year from leap month rejects");

TemporalHelpers.assertPlainDate(
  leap195205L.add(years1n),
  1951, 5, "M05", 29, "Subtracting 1 year from leap month M05L on day 30 constrains to M05 day 29"
);

assert.throws(RangeError, function () {
  leap195205L.add(years1n, options);
}, "Subtracting 1 year from leap month day 30 rejects");

TemporalHelpers.assertPlainDate(
  common202104.add(years1n, options),
  2020, 4, "M04", 1, "Subtracting 1 year from common-year M04 lands in leap-year M04"
);

TemporalHelpers.assertPlainDate(
  leap202004.add(years1n, options),
  2019, 4, "M04", 1, "Subtracting 1 year from leap-year M04 lands in common-year M04"
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2012, monthCode: "M03L", day: 1, calendar }, options).add(new Temporal.Duration(-19), options),
  1993, 4, "M03L", 1, "Subtracting years to go from one M03L to the previous M03L"
);

TemporalHelpers.assertPlainDate(
  common200108.add(years1n, options),
  2000, 8, "M08", 2, "Subtracting 1 year crossing leap month"
);

TemporalHelpers.assertPlainDate(
  common202104.add(new Temporal.Duration(-2), options),
  2019, 4, "M04", 1, "Subtracting 2 years from common-year M04 crossing leap year lands in common-year M04"
);

// Months

const months1 = new Temporal.Duration(0, 1);
const months1n = new Temporal.Duration(0, -1);
const months12 = new Temporal.Duration(0, 12);
const months12n = new Temporal.Duration(0, -12);
const months13 = new Temporal.Duration(0, 13);
const months13n = new Temporal.Duration(0, -13);

const leap202003 = Temporal.PlainDate.from({ year: 2020, monthCode: "M03", day: 1, calendar }, options);
const leap202006 = Temporal.PlainDate.from({ year: 2020, monthCode: "M06", day: 1, calendar }, options);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1947, monthCode: "M02L", day: 1, calendar }, options).add(months1),
  1947, 4, "M03", 1, "add 1 month, starting at start of leap month"
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1955, monthCode: "M03L", day: 1, calendar }, options).add(months1),
  1955, 5, "M04", 1, "add 1 month, starting at start of leap month with 30 days"
);

TemporalHelpers.assertPlainDate(
  leap202003.add(months1),
  2020, 4, "M04", 1, "adding 1 month to M03 in leap year lands in M04 (not M04L)"
);

TemporalHelpers.assertPlainDate(
  leap202003.add(new Temporal.Duration(0, 2)),
  2020, 5, "M04L", 1, "adding 2 months to M03 in leap year lands in M04L (leap month)"
);

TemporalHelpers.assertPlainDate(
  leap202003.add(new Temporal.Duration(0, 3)),
  2020, 6, "M05", 1, "adding 3 months to M03 in leap year lands in M05 (not M06)"
);

TemporalHelpers.assertPlainDate(
  common201904.add(months12),
  2020, 4, "M04", 1, "Adding 12 months to common-year M04 lands in leap-year M04"
);

TemporalHelpers.assertPlainDate(
  common201904.add(months13),
  2020, 5, "M04L", 1, "Adding 13 months to common-year M04 lands in leap-year M04L"
);

TemporalHelpers.assertPlainDate(
  leap202004.add(months12),
  2021, 3, "M03", 1, "Adding 12 months to leap-year M04 lands in common-year M03"
);

TemporalHelpers.assertPlainDate(
  leap202004.add(months13),
  2021, 4, "M04", 1, "Adding 13 months to leap-year M04 lands in common-year M04"
);

TemporalHelpers.assertPlainDate(
  leap202004L.add(months12),
  2021, 4, "M04", 1, "Adding 12 months to M04L lands in common-year M04"
);

TemporalHelpers.assertPlainDate(
  common200008.add(new Temporal.Duration(1, 12), options),
  2002, 8, "M08", 2, "Adding 1y 12mo crossing leap month in the year part"
);

TemporalHelpers.assertPlainDate(
  common200108.add(new Temporal.Duration(2, 13), options),
  2004, 9, "M08", 2, "Adding 1y 13mo crossing leap month in the months part"
);

TemporalHelpers.assertPlainDate(
  common201904.add(new Temporal.Duration(0, 24)),
  2021, 3, "M03", 1, "Adding 24 months to common-year M04 crossing leap year with M04L, lands in common-year M03"
);

TemporalHelpers.assertPlainDate(
  common201904.add(new Temporal.Duration(0, 25)),
  2021, 4, "M04", 1, "Adding 25 months to common-year M04 crossing leap year with M04L, lands in common-year M04"
);

TemporalHelpers.assertPlainDate(
  leap202006.add(months1n),
  2020, 6, "M05", 1, "Subtracting 1 month from M06 in leap year lands in M05"
);

TemporalHelpers.assertPlainDate(
  leap202006.add(new Temporal.Duration(0, -2)),
  2020, 5, "M04L", 1, "Subtracting 2 months from M06 in leap year lands in M04L (leap month)"
);

TemporalHelpers.assertPlainDate(
  leap202006.add(new Temporal.Duration(0, -3)),
  2020, 4, "M04", 1, "Subtracting 3 months from M06 in leap year lands in M04 (not M03)"
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2020, monthCode: "M05", day: 1, calendar }, options).add(months1n),
  2020, 5, "M04L", 1, "Subtracting 1 month from M05 in leap year lands in M04L"
);

TemporalHelpers.assertPlainDate(
  leap202004L.add(months1n),
  2020, 4, "M04", 1, "Subtracting 1 month from M04L in calendar lands in M04"
);

TemporalHelpers.assertPlainDate(
  common202104.add(months12n),
  2020, 5, "M04L", 1, "Subtracting 12 months from common-year M04 lands in leap-year M04L"
);

TemporalHelpers.assertPlainDate(
  common202104.add(months13n),
  2020, 4, "M04", 1, "Subtracting 13 months from common-year M04 lands in leap-year M04"
);

TemporalHelpers.assertPlainDate(
  leap202004.add(months12n),
  2019, 4, "M04", 1, "Subtracting 12 months from leap-year M04 lands in common-year M04"
);

TemporalHelpers.assertPlainDate(
  leap202004L.add(months12n),
  2019, 5, "M05", 1, "Subtracting 12 months from M04L lands in common-year M05"
);

TemporalHelpers.assertPlainDate(
  leap202004L.add(months13n),
  2019, 4, "M04", 1, "Subtracting 13 months from M04L lands in common-year M04"
);

TemporalHelpers.assertPlainDate(
  common200108.add(new Temporal.Duration(-1, -12), options),
  1999, 8, "M08", 2, "Adding 1y 12mo crossing leap month in the year part"
);

TemporalHelpers.assertPlainDate(
  common200008.add(new Temporal.Duration(-2, -13), options),
  1997, 8, "M08", 2, "Adding 1y 13mo crossing leap month in the months part"
);

TemporalHelpers.assertPlainDate(
  common202104.add(new Temporal.Duration(0, -24)),
  2019, 5, "M05", 1, "Subtracting 24 months from common-year M04 crossing leap year with M04L, lands in common-year M05"
);

TemporalHelpers.assertPlainDate(
  common202104.add(new Temporal.Duration(0, -25)),
  2019, 4, "M04", 1, "Subtracting 25 months from common-year M04 crossing leap year with M04L, lands in common-year M04"
);

// Weeks

const months2weeks3 = new Temporal.Duration(0, /* months = */ 2, /* weeks = */ 3);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1947, monthCode: "M02L", day: 29, calendar }, options).add(months2weeks3),
  1947, 6, "M05", 20, "add 2 months 3 weeks from last day leap month without leap day"
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1955, monthCode: "M03L", day: 30, calendar }, options).add(months2weeks3),
  1955, 7, "M06", 21, "add 2 months 3 weeks from leap day in leap month"
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1947, monthCode: "M01", day: 29, calendar }, options).add(months2weeks3),
  1947, 4, "M03", 21, "add 2 months 3 weeks from immediately before a leap month"
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1955, monthCode: "M06", day: 29, calendar }, options).add(months2weeks3),
  1955, 10, "M09", 20, "add 2 months 3 weeks from immediately before a leap month"
);

// Days

const days10 = new Temporal.Duration(0, 0, 0, /* days = */ 10);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1955, monthCode: "M03L", day: 30, calendar }, options).add(days10),
  1955, 5, "M04", 10, "add 10 days from leap day in leap month"
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1947, monthCode: "M02L", day: 29, calendar }, options).add(days10),
  1947, 4, "M03", 10, "add 10 days from last day of leap month"
);
