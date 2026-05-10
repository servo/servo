// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Arithmetic around leap months in the hebrew calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years2 = new Temporal.Duration(-2);
const years2n = new Temporal.Duration(2);

const leap1AdarI = Temporal.PlainYearMonth.from({ year: 5782, monthCode: "M05L", calendar }, options);
const leap1AdarII = Temporal.PlainYearMonth.from({ year: 5782, monthCode: "M06", calendar }, options);
const common1Adar = Temporal.PlainYearMonth.from({ year: 5783, monthCode: "M06", calendar }, options);
const common = Temporal.PlainYearMonth.from({ year: 5783, monthCode: "M08", calendar }, options);
const leap2AdarI = Temporal.PlainYearMonth.from({ year: 5784, monthCode: "M05L", calendar }, options);
const leap2AdarII = Temporal.PlainYearMonth.from({ year: 5784, monthCode: "M06", calendar }, options);
const common2Adar = Temporal.PlainYearMonth.from({ year: 5785, monthCode: "M06", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  common1Adar.subtract(years1, options),
  5784, 7, "M06", "Adding 1 year to common-year Adar (M06) lands in leap-year Adar II (M06)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  common1Adar.subtract(years2, options),
  5785, 6, "M06", "Adding 2 years to common-year Adar (M06) crossing leap year lands in common-year Adar (M06)",
  "am", 5785, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarI.subtract(years1),
  5785, 6, "M06", "Adding 1 year to Adar I (M05L) lands in common-year Adar (M06) with constrain",
  "am", 5785, null
);

assert.throws(RangeError, function () {
  leap2AdarI.subtract(years1, options);
}, "Adding 1 year to Adar I (M05L) rejects");

TemporalHelpers.assertPlainYearMonth(
  leap2AdarII.subtract(years1, options),
  5785, 6, "M06", "Adding 1 year to Adar II (M06) lands in common-year Adar (M06) even with reject",
  "am", 5785, null
);

TemporalHelpers.assertPlainYearMonth(
  common.subtract(years1, options),
  5784, 9, "M08", "Adding 1 year across Adar I (M05L)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  leap1AdarI.subtract(years2, options),
  5784, 6, "M05L", "Adding 2 years to leap-year Adar I (M05L) lands in leap-year Adar I (M05L)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  leap1AdarII.subtract(years2, options),
  5784, 7, "M06", "Adding 2 years to leap-year Adar II (M06) lands in leap-year Adar II (M06)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  common2Adar.subtract(years1n, options),
  5784, 7, "M06", "Subtracting 1 year from common-year Adar (M06) lands in leap-year Adar II (M06)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  common2Adar.subtract(years2n, options),
  5783, 6, "M06", "Subtracting 2 years from common-year Adar (M06) crossing leap year lands in common-year Adar (M06)",
  "am", 5783, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarI.subtract(years1n),
  5783, 6, "M06", "Subtracting 1 year from Adar I (M05L) lands in common-year Adar (M06) with constrain",
  "am", 5783, null
);

assert.throws(RangeError, function () {
  leap2AdarI.subtract(years1n, options);
}, "Subtracting 1 year from Adar I (M05L) rejects");

TemporalHelpers.assertPlainYearMonth(
  leap2AdarII.subtract(years1n, options),
  5783, 6, "M06", "Subtracting 1 year from Adar II (M06) lands in common-year Adar (M06) even with reject",
  "am", 5783, null
);

TemporalHelpers.assertPlainYearMonth(
  common.subtract(years2n, options),
  5781, 8, "M08", "Subtracting 2 years across Adar I (M05L)",
  "am", 5781, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarI.subtract(years2n, options),
  5782, 6, "M05L", "Subtracting 2 years from leap-year Adar I (M05L) lands in leap-year Adar I (M05L)",
  "am", 5782, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarII.subtract(years2n, options),
  5782, 7, "M06", "Subtracting 2 years from leap-year Adar II (M06) lands in leap-year Adar II (M06)",
  "am", 5782, null
);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months2 = new Temporal.Duration(0, -2);
const months2n = new Temporal.Duration(0, 2);
const months12 = new Temporal.Duration(0, -12);
const months12n = new Temporal.Duration(0, 12);
const months13 = new Temporal.Duration(0, -13);
const months13n = new Temporal.Duration(0, 13);
const months24 = new Temporal.Duration(0, -24);
const months24n = new Temporal.Duration(0, 24);

const date1 = Temporal.PlainYearMonth.from({ year: 5784, monthCode: "M04", calendar }, options);
const date3 = Temporal.PlainYearMonth.from({ year: 5784, monthCode: "M07", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date1.subtract(months1, options),
  5784, 5, "M05", "Adding 1 month to M04 in leap year lands in M05 (Shevat)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  date1.subtract(months2, options),
  5784, 6, "M05L", "Adding 2 months to M04 in leap year lands in M05L (Adar I)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  date1.subtract(new Temporal.Duration(0, -3), options),
  5784, 7, "M06", "Adding 3 months to M04 in leap year lands in M06 (Adar II)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarI.subtract(months1, options),
  5784, 7, "M06", "Adding 1 month to M05L (Adar I) lands in M06 (Adar II)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 5783, monthCode: "M04", calendar }, options).subtract(months2, options),
  5783, 6, "M06", "Adding 2 months to M04 in non-leap year lands in M06 (no M05L)",
  "am", 5783, null
);

TemporalHelpers.assertPlainYearMonth(
  common1Adar.subtract(months12, options),
  5784, 6, "M05L", "Adding 12 months to common-year Adar lands in leap-year Adar I (M05L)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  common1Adar.subtract(months13, options),
  5784, 7, "M06", "Adding 13 months to common-year Adar lands in leap-year Adar II (M06)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarI.subtract(months12, options),
  5785, 5, "M05", "Adding 12 months to leap-year Adar I lands in Shevat (M05)",
  "am", 5785, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarI.subtract(months13, options),
  5785, 6, "M06", "Adding 13 months to leap-year Adar I lands in Adar (M06)",
  "am", 5785, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarII.subtract(months12, options),
  5785, 6, "M06", "Adding 12 months to leap-year Adar II lands in Adar (M06)",
  "am", 5785, null
);

TemporalHelpers.assertPlainYearMonth(
  common.subtract(months13, options),
  5784, 9, "M08", "Adding 13 months across Adar I (M05L) lands in same month code",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  common.subtract(new Temporal.Duration(-1, -12), options),
  5785, 8, "M08", "Adding 1y 12mo across Adar I (M05L) in the years part lands in same month code",
  "am", 5785, null
);

TemporalHelpers.assertPlainYearMonth(
  date3.subtract(new Temporal.Duration(-2, -13), options),
  5787, 8, "M07", "Adding 2y 13mo across Adar I (M05L) in the months part lands in same month code",
  "am", 5787, null
);

TemporalHelpers.assertPlainYearMonth(
  common1Adar.subtract(months24, options),
  5785, 5, "M05", "Adding 24 months to common-year Adar crossing a leap year lands in common-year Shevat (M05)",
  "am", 5785, null
);

TemporalHelpers.assertPlainYearMonth(
  common1Adar.subtract(new Temporal.Duration(0, -25), options),
  5785, 6, "M06", "Adding 25 months to common-year Adar crossing a leap year lands in common-year Adar (M06)",
  "am", 5785, null
);

TemporalHelpers.assertPlainYearMonth(
  leap1AdarI.subtract(months24, options),
  5784, 5, "M05", "Adding 24 months to leap-year Adar I lands in leap-year Shevat (M05)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  leap1AdarII.subtract(months24, options),
  5784, 6, "M05L", "Adding 24 months to leap-year Adar II lands in leap-year Adar I (M05L)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  date3.subtract(months1n, options),
  5784, 7, "M06", "Subtracting 1 month from M07 in leap year lands in M06 (Adar II)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  date3.subtract(months2n, options),
  5784, 6, "M05L", "Subtracting 2 months from M07 in leap year lands in M05L (Adar I)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  date3.subtract(new Temporal.Duration(0, 3), options),
  5784, 5, "M05", "Subtracting 3 months from M07 in leap year lands in M05 (Shevat)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarII.subtract(months1n, options),
  5784, 6, "M05L", "Subtracting 1 month from M06 (Adar II) in leap year lands in M05L (Adar I)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarI.subtract(months1n, options),
  5784, 5, "M05", "Subtracting 1 month from M05L (Adar I) lands in M05 (Shevat)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 5783, monthCode: "M07", calendar }).subtract(months2n, options),
  5783, 5, "M05", "Subtracting 2 months from M07 in non-leap year lands in M05 (no M05L)",
  "am", 5783, null
);

TemporalHelpers.assertPlainYearMonth(
  common2Adar.subtract(months12n, options),
  5784, 7, "M06", "Subtracting 12 months from common-year Adar lands in leap-year Adar II (M06)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  common2Adar.subtract(months13n, options),
  5784, 6, "M05L", "Subtracting 13 months from common-year Adar lands in leap-year Adar I (M05L)",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarI.subtract(months12n, options),
  5783, 6, "M06", "Subtracting 12 months from leap-year Adar I lands in Adar (M06)",
  "am", 5783, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarI.subtract(months13n, options),
  5783, 5, "M05", "Subtracting 13 months from leap-year Adar I lands in Shevat (M05)",
  "am", 5783, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarII.subtract(months12n, options),
  5783, 7, "M07", "Subtracting 12 months from leap-year Adar II lands in Nisan (M07)",
  "am", 5783, null
);

TemporalHelpers.assertPlainYearMonth(
  common2Adar.subtract(months24n, options),
  5783, 7, "M07", "Subtracting 24 months from common-year Adar crossing a leap year lands in common-year Nisan (M07)",
  "am", 5783, null
);

TemporalHelpers.assertPlainYearMonth(
  date1.subtract(new Temporal.Duration(2, 12), options),
  5781, 4, "M04", "Subtracting 2y 12mo across Adar I (M05L) in the years part lands in same month code",
  "am", 5781, null
);

TemporalHelpers.assertPlainYearMonth(
  date1.subtract(new Temporal.Duration(1, 13), options),
  5782, 4, "M04", "Subtracting 1y 13mo across Adar I (M05L) in the months part lands in same month code",
  "am", 5782, null
);

TemporalHelpers.assertPlainYearMonth(
  common2Adar.subtract(new Temporal.Duration(0, 25), options),
  5783, 6, "M06", "Subtracting 25 months from common-year Adar crossing a leap year lands in common-year Adar (M06)",
  "am", 5783, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarI.subtract(months24n, options),
  5782, 7, "M06", "Subtracting 24 months from leap-year Adar I lands in leap-year Adar (M06)",
  "am", 5782, null
);

TemporalHelpers.assertPlainYearMonth(
  leap2AdarII.subtract(months24n, options),
  5782, 8, "M07", "Subtracting 24 months from leap-year Adar II lands in leap-year Nisan (M07)",
  "am", 5782, null
);
