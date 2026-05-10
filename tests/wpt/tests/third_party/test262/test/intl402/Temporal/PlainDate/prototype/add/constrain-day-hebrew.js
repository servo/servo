// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Constraining the day at end of month (hebrew calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

// 30-day months: 01, 05, 05L, 07, 09, 11
// 29-day months: 04, 06, 08, 10, 12
//
// Cheshvan and Kislev (02, 03) have 29 or 30 days, independent of leap years.
// Deficient - Cheshvan and Kislev have 29 days
// Regular - Cheshvan has 29 days, Kislev 30
// Complete - Cheshvan and Kislev have 30 days
//
// Some recent years of each type: 
// 5778 - regular common year
// 5779 - complete leap year
// 5781 - deficient common year
// 5782 - regular leap year
// 5783 - complete common year
// 5784 - deficient leap year

// Years

const rcM03 = Temporal.PlainDate.from({ year: 5778, monthCode: "M03", day: 30, calendar }, options);
const rlM03 = Temporal.PlainDate.from({ year: 5782, monthCode: "M03", day: 30, calendar }, options);
const ccM02 = Temporal.PlainDate.from({ year: 5783, monthCode: "M02", day: 30, calendar }, options);
const ccM03 = Temporal.PlainDate.from({ year: 5783, monthCode: "M03", day: 30, calendar }, options);
const clM02 = Temporal.PlainDate.from({ year: 5779, monthCode: "M02", day: 30, calendar }, options);
const clM03 = Temporal.PlainDate.from({ year: 5779, monthCode: "M03", day: 30, calendar }, options);

const years1n = new Temporal.Duration(-1);
const years2n = new Temporal.Duration(-2);
const years2 = new Temporal.Duration(2);
const years3 = new Temporal.Duration(3);

TemporalHelpers.assertPlainDate(
  rcM03.add(years3),
  5781, 3, "M03", 29, "regular-year Kislev constrains to 29 in deficient year when adding",
  "am", 5781);
assert.throws(RangeError, function () {
  rcM03.add(years3, options);
}, "regular-year Kislev rejects 30 in deficient year when adding");

TemporalHelpers.assertPlainDate(
  clM02.add(years3),
  5782, 2, "M02", 29, "complete-year Cheshvan constrains to 29 in regular year when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  clM02.add(years3, options);
}, "complete-year Cheshvan rejects 30 in regular year when adding");

TemporalHelpers.assertPlainDate(
  clM02.add(years2),
  5781, 2, "M02", 29, "complete-year Cheshvan constrains to 29 in deficient year when adding",
  "am", 5781);
assert.throws(RangeError, function () {
  clM02.add(years2, options);
}, "complete-year Cheshvan rejects 30 in deficient year when adding");

TemporalHelpers.assertPlainDate(
  clM03.add(years2),
  5781, 3, "M03", 29, "complete-year Kislev constrains to 29 in deficient year when adding",
  "am", 5781);
assert.throws(RangeError, function () {
  clM03.add(years2, options);
}, "complete-year Kislev rejects 30 in deficient year when adding");

TemporalHelpers.assertPlainDate(
  rlM03.add(years1n),
  5781, 3, "M03", 29, "regular-year Kislev constrains to 29 in deficient year when subtracting",
  "am", 5781);
assert.throws(RangeError, function () {
  rlM03.add(years1n, options);
}, "regular-year Kislev rejects 30 in deficient year when subtracting");

TemporalHelpers.assertPlainDate(
  ccM02.add(years1n),
  5782, 2, "M02", 29, "complete-year Cheshvan constrains to 29 in regular year when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  ccM02.add(years1n, options);
}, "complete-year Cheshvan rejects 30 in regular year when subtracting");

TemporalHelpers.assertPlainDate(
  ccM02.add(years2n),
  5781, 2, "M02", 29, "complete-year Cheshvan constrains to 29 in deficient year when subtracting",
  "am", 5781);
assert.throws(RangeError, function () {
  ccM02.add(years2n, options);
}, "complete-year Cheshvan rejects 30 in deficient year when subtracting");

TemporalHelpers.assertPlainDate(
  ccM03.add(years2n),
  5781, 3, "M03", 29, "complete-year Kislev constrains to 29 in deficient year when subtracting",
  "am", 5781);
assert.throws(RangeError, function () {
  ccM03.add(years2n, options);
}, "complete-year Kislev rejects 30 in deficient year when subtracting");

// Months

const dc0130 = Temporal.PlainDate.from({ year: 5781, monthCode: "M01", day: 30, calendar }, options);
const rc0130 = Temporal.PlainDate.from({ year: 5778, monthCode: "M01", day: 30, calendar }, options);
const cc0130 = Temporal.PlainDate.from({ year: 5783, monthCode: "M01", day: 30, calendar }, options);
const dl0130 = Temporal.PlainDate.from({ year: 5784, monthCode: "M01", day: 30, calendar }, options);
const rl0130 = Temporal.PlainDate.from({ year: 5782, monthCode: "M01", day: 30, calendar }, options);
const cl0130 = Temporal.PlainDate.from({ year: 5779, monthCode: "M01", day: 30, calendar }, options);
// 01-30 of each of the above years plus 1, for subtracting
const dc0130After = Temporal.PlainDate.from({ year: 5782, monthCode: "M01", day: 30, calendar }, options);
const rc0130After = Temporal.PlainDate.from({ year: 5779, monthCode: "M01", day: 30, calendar }, options);
const cc0130After = Temporal.PlainDate.from({ year: 5784, monthCode: "M01", day: 30, calendar }, options);
const dl0130After = Temporal.PlainDate.from({ year: 5785, monthCode: "M01", day: 30, calendar }, options);
const rl0130After = Temporal.PlainDate.from({ year: 5783, monthCode: "M01", day: 30, calendar }, options);
const cl0130After = Temporal.PlainDate.from({ year: 5780, monthCode: "M01", day: 30, calendar }, options);

const months1 = new Temporal.Duration(0, 1);
const months2 = new Temporal.Duration(0, 2);
const months3 = new Temporal.Duration(0, 3);
const months4 = new Temporal.Duration(0, 4);
const months5 = new Temporal.Duration(0, 5);
const months6 = new Temporal.Duration(0, 6);
const months7 = new Temporal.Duration(0, 7);
const months8 = new Temporal.Duration(0, 8);
const months9 = new Temporal.Duration(0, 9);
const months10 = new Temporal.Duration(0, 10);
const months11 = new Temporal.Duration(0, 11);
const months12 = new Temporal.Duration(0, 12);
const months1n = new Temporal.Duration(0, -1);
const months2n = new Temporal.Duration(0, -2);
const months3n = new Temporal.Duration(0, -3);
const months4n = new Temporal.Duration(0, -4);
const months5n = new Temporal.Duration(0, -5);
const months6n = new Temporal.Duration(0, -6);
const months7n = new Temporal.Duration(0, -7);
const months8n = new Temporal.Duration(0, -8);
const months9n = new Temporal.Duration(0, -9);
const months10n = new Temporal.Duration(0, -10);
const months11n = new Temporal.Duration(0, -11);
const months12n = new Temporal.Duration(0, -12);
const months13n = new Temporal.Duration(0, -13);

// Common year, forwards

TemporalHelpers.assertPlainDate(
  rc0130.add(months1),
  5778, 2, "M02", 29, "regular common year Cheshvan constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months1, options);
}, "regular common year Cheshvan rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  cc0130.add(months1, options),
  5783, 2, "M02", 30, "regular common year Cheshvan does not reject 30 when adding",
  "am", 5783);

TemporalHelpers.assertPlainDate(
  rc0130.add(months2, options),
  5778, 3, "M03", 30, "regular common year Kislev does not reject 30 when adding",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  dc0130.add(months2),
  5781, 3, "M03", 29, "deficient common year Kislev constrains to 29 when adding",
  "am", 5781);
assert.throws(RangeError, function () {
  dc0130.add(months2, options);
}, "deficient common year Kislev rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  rc0130.add(months3),
  5778, 4, "M04", 29, "common-year Tevet constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months3, options);
}, "common-year Tevet rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  rc0130.add(months4, options),
  5778, 5, "M05", 30, "common-year Shevat does not reject 30 when adding",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130.add(months5),
  5778, 6, "M06", 29, "common-year Adar constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months5, options);
}, "common-year Adar rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  rc0130.add(months6, options),
  5778, 7, "M07", 30, "common-year Nisan does not reject 30 when adding",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130.add(months7),
  5778, 8, "M08", 29, "common-year Iyar constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months7, options);
}, "common-year Iyar rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  rc0130.add(months8, options),
  5778, 9, "M09", 30, "common-year Sivan does not reject 30 when adding",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130.add(months9),
  5778, 10, "M10", 29, "common-year Tammuz constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months9, options);
}, "common-year Tammuz rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  rc0130.add(months10, options),
  5778, 11, "M11", 30, "common-year Av does not reject 30 when adding",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130.add(months11),
  5778, 12, "M12", 29, "common-year Elul constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months11, options);
}, "common-year Elul rejects 30 when adding");

// Leap year, forwards

TemporalHelpers.assertPlainDate(
  rl0130.add(months1),
  5782, 2, "M02", 29, "regular leap year Cheshvan constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months1, options);
}, "regular leap year Cheshvan rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  cl0130.add(months1, options),
  5779, 2, "M02", 30, "regular leap year Cheshvan does not reject 30 when adding",
  "am", 5779);

TemporalHelpers.assertPlainDate(
  rl0130.add(months2, options),
  5782, 3, "M03", 30, "regular leap year Kislev does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  dl0130.add(months2),
  5784, 3, "M03", 29, "deficient leap year Kislev constrains to 29 when adding",
  "am", 5784);
assert.throws(RangeError, function () {
  dl0130.add(months2, options);
}, "deficient leap year Kislev rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  rl0130.add(months3),
  5782, 4, "M04", 29, "leap-year Tevet constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months3, options);
}, "leap-year Tevet rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  rl0130.add(months4, options),
  5782, 5, "M05", 30, "leap-year Shevat does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130.add(months5, options),
  5782, 6, "M05L", 30, "leap-year Adar I does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130.add(months6),
  5782, 7, "M06", 29, "leap-year Adar II constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months6, options);
}, "leap-year Adar II rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  rl0130.add(months7, options),
  5782, 8, "M07", 30, "leap-year Nisan does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130.add(months8),
  5782, 9, "M08", 29, "leap-year Iyar constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months8, options);
}, "leap-year Iyar rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  rl0130.add(months9, options),
  5782, 10, "M09", 30, "leap-year Sivan does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130.add(months10),
  5782, 11, "M10", 29, "leap-year Tammuz constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months10, options);
}, "leap-year Tammuz rejects 30 when adding");

TemporalHelpers.assertPlainDate(
  rl0130.add(months11, options),
  5782, 12, "M11", 30, "leap-year Av does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130.add(months12),
  5782, 13, "M12", 29, "leap-year Elul constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months12, options);
}, "leap-year Elul rejects 30 when adding");

// Common year, backwards

TemporalHelpers.assertPlainDate(
  rc0130After.add(months12n, options),
  5778, 1, "M01", 30, "common-year Tishrei does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130After.add(months11n),
  5778, 2, "M02", 29, "regular common year Cheshvan constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months11n, options);
}, "regular common year Cheshvan rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  cc0130After.add(months11n, options),
  5783, 2, "M02", 30, "regular common year Cheshvan does not reject 30 when subtracting",
  "am", 5783);

TemporalHelpers.assertPlainDate(
  rc0130After.add(months10n, options),
  5778, 3, "M03", 30, "regular common year Kislev does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  dc0130After.add(months10n),
  5781, 3, "M03", 29, "deficient common year Kislev constrains to 29 when subtracting",
  "am", 5781);
assert.throws(RangeError, function () {
  dc0130After.add(months10n, options);
}, "deficient common year Kislev rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  rc0130After.add(months9n),
  5778, 4, "M04", 29, "common-year Tevet constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months9n, options);
}, "common-year Tevet rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  rc0130After.add(months8n, options),
  5778, 5, "M05", 30, "common-year Shevat does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130After.add(months7n),
  5778, 6, "M06", 29, "common-year Adar constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months7n, options);
}, "common-year Adar rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  rc0130After.add(months6n, options),
  5778, 7, "M07", 30, "common-year Nisan does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130After.add(months5n),
  5778, 8, "M08", 29, "common-year Iyar constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months5n, options);
}, "common-year Iyar rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  rc0130After.add(months4n, options),
  5778, 9, "M09", 30, "common-year Sivan does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130After.add(months3n),
  5778, 10, "M10", 29, "common-year Tammuz constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months3n, options);
}, "common-year Tammuz rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  rc0130After.add(months2n, options),
  5778, 11, "M11", 30, "common-year Av does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130After.add(months1n),
  5778, 12, "M12", 29, "common-year Elul constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months1n, options);
}, "common-year Elul rejects 30 when subtracting");

// Leap year, backwards

TemporalHelpers.assertPlainDate(
  rl0130After.add(months13n, options),
  5782, 1, "M01", 30, "leap-year Tishrei does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130After.add(months12n),
  5782, 2, "M02", 29, "regular leap year Cheshvan constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months12n, options);
}, "regular leap year Cheshvan rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  cl0130After.add(months12n, options),
  5779, 2, "M02", 30, "regular leap year Cheshvan does not reject 30 when subtracting",
  "am", 5779);

TemporalHelpers.assertPlainDate(
  rl0130After.add(months11n, options),
  5782, 3, "M03", 30, "regular leap year Kislev does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  dl0130After.add(months11n),
  5784, 3, "M03", 29, "deficient leap year Kislev constrains to 29 when subtracting",
  "am", 5784);
assert.throws(RangeError, function () {
  dl0130After.add(months11n, options);
}, "deficient leap year Kislev rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  rl0130After.add(months10n),
  5782, 4, "M04", 29, "leap-year Tevet constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months10n, options);
}, "leap-year Tevet rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  rl0130After.add(months9n, options),
  5782, 5, "M05", 30, "leap-year Shevat does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130After.add(months8n, options),
  5782, 6, "M05L", 30, "leap-year Adar I does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130After.add(months7n),
  5782, 7, "M06", 29, "leap-year Adar II constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months7n, options);
}, "leap-year Adar II rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  rl0130After.add(months6n, options),
  5782, 8, "M07", 30, "leap-year Nisan does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130After.add(months5n),
  5782, 9, "M08", 29, "leap-year Iyar constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months5n, options);
}, "leap-year Iyar rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  rl0130After.add(months4n, options),
  5782, 10, "M09", 30, "leap-year Sivan does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130After.add(months3n),
  5782, 11, "M10", 29, "leap-year Tammuz constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months3n, options);
}, "leap-year Tammuz rejects 30 when subtracting");

TemporalHelpers.assertPlainDate(
  rl0130After.add(months2n, options),
  5782, 12, "M11", 30, "leap-year Av does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130After.add(months1n),
  5782, 13, "M12", 29, "leap-year Elul constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months1n, options);
}, "leap-year Elul rejects 30 when subtracting");
