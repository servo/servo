// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
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

const rcM03 = Temporal.ZonedDateTime.from({ year: 5778, monthCode: "M03", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const rlM03 = Temporal.ZonedDateTime.from({ year: 5782, monthCode: "M03", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const ccM02 = Temporal.ZonedDateTime.from({ year: 5783, monthCode: "M02", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const ccM03 = Temporal.ZonedDateTime.from({ year: 5783, monthCode: "M03", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const clM02 = Temporal.ZonedDateTime.from({ year: 5779, monthCode: "M02", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const clM03 = Temporal.ZonedDateTime.from({ year: 5779, monthCode: "M03", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

const years1n = new Temporal.Duration(-1);
const years2n = new Temporal.Duration(-2);
const years2 = new Temporal.Duration(2);
const years3 = new Temporal.Duration(3);

TemporalHelpers.assertPlainDateTime(
  rcM03.add(years3).toPlainDateTime(),
  5781, 3, "M03", 29, 12, 34, 0, 0, 0, 0, "regular-year Kislev constrains to 29 in deficient year when adding",
  "am", 5781);
assert.throws(RangeError, function () {
  rcM03.add(years3, options);
}, "regular-year Kislev rejects 30 in deficient year when adding");

TemporalHelpers.assertPlainDateTime(
  clM02.add(years3).toPlainDateTime(),
  5782, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "complete-year Cheshvan constrains to 29 in regular year when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  clM02.add(years3, options);
}, "complete-year Cheshvan rejects 30 in regular year when adding");

TemporalHelpers.assertPlainDateTime(
  clM02.add(years2).toPlainDateTime(),
  5781, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "complete-year Cheshvan constrains to 29 in deficient year when adding",
  "am", 5781);
assert.throws(RangeError, function () {
  clM02.add(years2, options);
}, "complete-year Cheshvan rejects 30 in deficient year when adding");

TemporalHelpers.assertPlainDateTime(
  clM03.add(years2).toPlainDateTime(),
  5781, 3, "M03", 29, 12, 34, 0, 0, 0, 0, "complete-year Kislev constrains to 29 in deficient year when adding",
  "am", 5781);
assert.throws(RangeError, function () {
  clM03.add(years2, options);
}, "complete-year Kislev rejects 30 in deficient year when adding");

TemporalHelpers.assertPlainDateTime(
  rlM03.add(years1n).toPlainDateTime(),
  5781, 3, "M03", 29, 12, 34, 0, 0, 0, 0, "regular-year Kislev constrains to 29 in deficient year when subtracting",
  "am", 5781);
assert.throws(RangeError, function () {
  rlM03.add(years1n, options);
}, "regular-year Kislev rejects 30 in deficient year when subtracting");

TemporalHelpers.assertPlainDateTime(
  ccM02.add(years1n).toPlainDateTime(),
  5782, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "complete-year Cheshvan constrains to 29 in regular year when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  ccM02.add(years1n, options);
}, "complete-year Cheshvan rejects 30 in regular year when subtracting");

TemporalHelpers.assertPlainDateTime(
  ccM02.add(years2n).toPlainDateTime(),
  5781, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "complete-year Cheshvan constrains to 29 in deficient year when subtracting",
  "am", 5781);
assert.throws(RangeError, function () {
  ccM02.add(years2n, options);
}, "complete-year Cheshvan rejects 30 in deficient year when subtracting");

TemporalHelpers.assertPlainDateTime(
  ccM03.add(years2n).toPlainDateTime(),
  5781, 3, "M03", 29, 12, 34, 0, 0, 0, 0, "complete-year Kislev constrains to 29 in deficient year when subtracting",
  "am", 5781);
assert.throws(RangeError, function () {
  ccM03.add(years2n, options);
}, "complete-year Kislev rejects 30 in deficient year when subtracting");

// Months

const dc0130 = Temporal.ZonedDateTime.from({ year: 5781, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const rc0130 = Temporal.ZonedDateTime.from({ year: 5778, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const cc0130 = Temporal.ZonedDateTime.from({ year: 5783, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const dl0130 = Temporal.ZonedDateTime.from({ year: 5784, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const rl0130 = Temporal.ZonedDateTime.from({ year: 5782, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const cl0130 = Temporal.ZonedDateTime.from({ year: 5779, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
// 01-30 of each of the above years plus 1, for subtracting
const dc0130After = Temporal.ZonedDateTime.from({ year: 5782, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const rc0130After = Temporal.ZonedDateTime.from({ year: 5779, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const cc0130After = Temporal.ZonedDateTime.from({ year: 5784, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const dl0130After = Temporal.ZonedDateTime.from({ year: 5785, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const rl0130After = Temporal.ZonedDateTime.from({ year: 5783, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const cl0130After = Temporal.ZonedDateTime.from({ year: 5780, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

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

TemporalHelpers.assertPlainDateTime(
  rc0130.add(months1).toPlainDateTime(),
  5778, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "regular common year Cheshvan constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months1, options);
}, "regular common year Cheshvan rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  cc0130.add(months1, options).toPlainDateTime(),
  5783, 2, "M02", 30, 12, 34, 0, 0, 0, 0, "regular common year Cheshvan does not reject 30 when adding",
  "am", 5783);

TemporalHelpers.assertPlainDateTime(
  rc0130.add(months2, options).toPlainDateTime(),
  5778, 3, "M03", 30, 12, 34, 0, 0, 0, 0, "regular common year Kislev does not reject 30 when adding",
  "am", 5778);

TemporalHelpers.assertPlainDateTime(
  dc0130.add(months2).toPlainDateTime(),
  5781, 3, "M03", 29, 12, 34, 0, 0, 0, 0, "deficient common year Kislev constrains to 29 when adding",
  "am", 5781);
assert.throws(RangeError, function () {
  dc0130.add(months2, options);
}, "deficient common year Kislev rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  rc0130.add(months3).toPlainDateTime(),
  5778, 4, "M04", 29, 12, 34, 0, 0, 0, 0, "common-year Tevet constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months3, options);
}, "common-year Tevet rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  rc0130.add(months4, options).toPlainDateTime(),
  5778, 5, "M05", 30, 12, 34, 0, 0, 0, 0, "common-year Shevat does not reject 30 when adding",
  "am", 5778);

TemporalHelpers.assertPlainDateTime(
  rc0130.add(months5).toPlainDateTime(),
  5778, 6, "M06", 29, 12, 34, 0, 0, 0, 0, "common-year Adar constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months5, options);
}, "common-year Adar rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  rc0130.add(months6, options).toPlainDateTime(),
  5778, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "common-year Nisan does not reject 30 when adding",
  "am", 5778);

TemporalHelpers.assertPlainDateTime(
  rc0130.add(months7).toPlainDateTime(),
  5778, 8, "M08", 29, 12, 34, 0, 0, 0, 0, "common-year Iyar constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months7, options);
}, "common-year Iyar rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  rc0130.add(months8, options).toPlainDateTime(),
  5778, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "common-year Sivan does not reject 30 when adding",
  "am", 5778);

TemporalHelpers.assertPlainDateTime(
  rc0130.add(months9).toPlainDateTime(),
  5778, 10, "M10", 29, 12, 34, 0, 0, 0, 0, "common-year Tammuz constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months9, options);
}, "common-year Tammuz rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  rc0130.add(months10, options).toPlainDateTime(),
  5778, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "common-year Av does not reject 30 when adding",
  "am", 5778);

TemporalHelpers.assertPlainDateTime(
  rc0130.add(months11).toPlainDateTime(),
  5778, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "common-year Elul constrains to 29 when adding",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.add(months11, options);
}, "common-year Elul rejects 30 when adding");

// Leap year, forwards

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months1).toPlainDateTime(),
  5782, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "regular leap year Cheshvan constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months1, options);
}, "regular leap year Cheshvan rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  cl0130.add(months1, options).toPlainDateTime(),
  5779, 2, "M02", 30, 12, 34, 0, 0, 0, 0, "regular leap year Cheshvan does not reject 30 when adding",
  "am", 5779);

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months2, options).toPlainDateTime(),
  5782, 3, "M03", 30, 12, 34, 0, 0, 0, 0, "regular leap year Kislev does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  dl0130.add(months2).toPlainDateTime(),
  5784, 3, "M03", 29, 12, 34, 0, 0, 0, 0, "deficient leap year Kislev constrains to 29 when adding",
  "am", 5784);
assert.throws(RangeError, function () {
  dl0130.add(months2, options);
}, "deficient leap year Kislev rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months3).toPlainDateTime(),
  5782, 4, "M04", 29, 12, 34, 0, 0, 0, 0, "leap-year Tevet constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months3, options);
}, "leap-year Tevet rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months4, options).toPlainDateTime(),
  5782, 5, "M05", 30, 12, 34, 0, 0, 0, 0, "leap-year Shevat does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months5, options).toPlainDateTime(),
  5782, 6, "M05L", 30, 12, 34, 0, 0, 0, 0, "leap-year Adar I does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months6).toPlainDateTime(),
  5782, 7, "M06", 29, 12, 34, 0, 0, 0, 0, "leap-year Adar II constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months6, options);
}, "leap-year Adar II rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months7, options).toPlainDateTime(),
  5782, 8, "M07", 30, 12, 34, 0, 0, 0, 0, "leap-year Nisan does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months8).toPlainDateTime(),
  5782, 9, "M08", 29, 12, 34, 0, 0, 0, 0, "leap-year Iyar constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months8, options);
}, "leap-year Iyar rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months9, options).toPlainDateTime(),
  5782, 10, "M09", 30, 12, 34, 0, 0, 0, 0, "leap-year Sivan does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months10).toPlainDateTime(),
  5782, 11, "M10", 29, 12, 34, 0, 0, 0, 0, "leap-year Tammuz constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months10, options);
}, "leap-year Tammuz rejects 30 when adding");

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months11, options).toPlainDateTime(),
  5782, 12, "M11", 30, 12, 34, 0, 0, 0, 0, "leap-year Av does not reject 30 when adding",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  rl0130.add(months12).toPlainDateTime(),
  5782, 13, "M12", 29, 12, 34, 0, 0, 0, 0, "leap-year Elul constrains to 29 when adding",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.add(months12, options);
}, "leap-year Elul rejects 30 when adding");

// Common year, backwards

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months12n, options).toPlainDateTime(),
  5778, 1, "M01", 30, 12, 34, 0, 0, 0, 0, "common-year Tishrei does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months11n).toPlainDateTime(),
  5778, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "regular common year Cheshvan constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months11n, options);
}, "regular common year Cheshvan rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  cc0130After.add(months11n, options).toPlainDateTime(),
  5783, 2, "M02", 30, 12, 34, 0, 0, 0, 0, "regular common year Cheshvan does not reject 30 when subtracting",
  "am", 5783);

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months10n, options).toPlainDateTime(),
  5778, 3, "M03", 30, 12, 34, 0, 0, 0, 0, "regular common year Kislev does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDateTime(
  dc0130After.add(months10n).toPlainDateTime(),
  5781, 3, "M03", 29, 12, 34, 0, 0, 0, 0, "deficient common year Kislev constrains to 29 when subtracting",
  "am", 5781);
assert.throws(RangeError, function () {
  dc0130After.add(months10n, options);
}, "deficient common year Kislev rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months9n).toPlainDateTime(),
  5778, 4, "M04", 29, 12, 34, 0, 0, 0, 0, "common-year Tevet constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months9n, options);
}, "common-year Tevet rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months8n, options).toPlainDateTime(),
  5778, 5, "M05", 30, 12, 34, 0, 0, 0, 0, "common-year Shevat does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months7n).toPlainDateTime(),
  5778, 6, "M06", 29, 12, 34, 0, 0, 0, 0, "common-year Adar constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months7n, options);
}, "common-year Adar rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months6n, options).toPlainDateTime(),
  5778, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "common-year Nisan does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months5n).toPlainDateTime(),
  5778, 8, "M08", 29, 12, 34, 0, 0, 0, 0, "common-year Iyar constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months5n, options);
}, "common-year Iyar rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months4n, options).toPlainDateTime(),
  5778, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "common-year Sivan does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months3n).toPlainDateTime(),
  5778, 10, "M10", 29, 12, 34, 0, 0, 0, 0, "common-year Tammuz constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months3n, options);
}, "common-year Tammuz rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months2n, options).toPlainDateTime(),
  5778, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "common-year Av does not reject 30 when subtracting",
  "am", 5778);

TemporalHelpers.assertPlainDateTime(
  rc0130After.add(months1n).toPlainDateTime(),
  5778, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "common-year Elul constrains to 29 when subtracting",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130After.add(months1n, options);
}, "common-year Elul rejects 30 when subtracting");

// Leap year, backwards

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months13n, options).toPlainDateTime(),
  5782, 1, "M01", 30, 12, 34, 0, 0, 0, 0, "leap-year Tishrei does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months12n).toPlainDateTime(),
  5782, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "regular leap year Cheshvan constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months12n, options);
}, "regular leap year Cheshvan rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  cl0130After.add(months12n, options).toPlainDateTime(),
  5779, 2, "M02", 30, 12, 34, 0, 0, 0, 0, "regular leap year Cheshvan does not reject 30 when subtracting",
  "am", 5779);

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months11n, options).toPlainDateTime(),
  5782, 3, "M03", 30, 12, 34, 0, 0, 0, 0, "regular leap year Kislev does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  dl0130After.add(months11n).toPlainDateTime(),
  5784, 3, "M03", 29, 12, 34, 0, 0, 0, 0, "deficient leap year Kislev constrains to 29 when subtracting",
  "am", 5784);
assert.throws(RangeError, function () {
  dl0130After.add(months11n, options);
}, "deficient leap year Kislev rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months10n).toPlainDateTime(),
  5782, 4, "M04", 29, 12, 34, 0, 0, 0, 0, "leap-year Tevet constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months10n, options);
}, "leap-year Tevet rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months9n, options).toPlainDateTime(),
  5782, 5, "M05", 30, 12, 34, 0, 0, 0, 0, "leap-year Shevat does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months8n, options).toPlainDateTime(),
  5782, 6, "M05L", 30, 12, 34, 0, 0, 0, 0, "leap-year Adar I does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months7n).toPlainDateTime(),
  5782, 7, "M06", 29, 12, 34, 0, 0, 0, 0, "leap-year Adar II constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months7n, options);
}, "leap-year Adar II rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months6n, options).toPlainDateTime(),
  5782, 8, "M07", 30, 12, 34, 0, 0, 0, 0, "leap-year Nisan does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months5n).toPlainDateTime(),
  5782, 9, "M08", 29, 12, 34, 0, 0, 0, 0, "leap-year Iyar constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months5n, options);
}, "leap-year Iyar rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months4n, options).toPlainDateTime(),
  5782, 10, "M09", 30, 12, 34, 0, 0, 0, 0, "leap-year Sivan does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months3n).toPlainDateTime(),
  5782, 11, "M10", 29, 12, 34, 0, 0, 0, 0, "leap-year Tammuz constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months3n, options);
}, "leap-year Tammuz rejects 30 when subtracting");

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months2n, options).toPlainDateTime(),
  5782, 12, "M11", 30, 12, 34, 0, 0, 0, 0, "leap-year Av does not reject 30 when subtracting",
  "am", 5782);

TemporalHelpers.assertPlainDateTime(
  rl0130After.add(months1n).toPlainDateTime(),
  5782, 13, "M12", 29, 12, 34, 0, 0, 0, 0, "leap-year Elul constrains to 29 when subtracting",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130After.add(months1n, options);
}, "leap-year Elul rejects 30 when subtracting");
