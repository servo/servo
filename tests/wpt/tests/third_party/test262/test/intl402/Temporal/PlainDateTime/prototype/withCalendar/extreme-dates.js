// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.withcalendar
description: withCalendar gives sensible output at extremes of supported range
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const min = new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1);
const max = new Temporal.PlainDateTime(275760, 9, 13, 23, 59, 59, 999, 999, 999);

// Lunisolar/lunar calendars can't accurately predict celestial orbits for dates
// far into the past/future. Skip `chinese` and `dangi`. `islamic-umalqura` is
// okay because it is specified to fall back to `islamic-civil` outside the
// range of accuracy.

const testData = [
  ["buddhist", -271278, 4, "M04", 19, "be", -271278, 276303, 9, "M09", 13, "be", 276303],
  ["coptic", -272099, 3, "M03", 23, "am", -272099, 275471, 5, "M05", 22, "am", 275471],
  ["ethioaa", -266323, 3, "M03", 23, "aa", -266323, 281247, 5, "M05", 22, "aa", 281247],
  ["ethiopic", -271823, 3, "M03", 23, "aa", -266323, 275747, 5, "M05", 22, "am", 275747],
  ["gregory", -271821, 4, "M04", 19, "bce", 271822, 275760, 9, "M09", 13, "ce", 275760],
  ["hebrew", -268058, 11, "M11", 4, "am", -268058, 279517, 10, "M09", 11, "am", 279517],
  ["indian", -271899, 1, "M01", 29, "shaka", -271899, 275682, 6, "M06", 22, "shaka", 275682],
  ["islamic-civil", -280804, 3, "M03", 21, "bh", 280805, 283583, 5, "M05", 23, "ah", 283583],
  ["islamic-tbla", -280804, 3, "M03", 22, "bh", 280805, 283583, 5, "M05", 24, "ah", 283583],
  ["islamic-umalqura", -280804, 3, "M03", 21, "bh", 280805, 283583, 5, "M05", 23, "ah", 283583],
  ["japanese", -271821, 4, "M04", 19, "bce", 271822, 275760, 9, "M09", 13, "reiwa", 273742],
  ["persian", -272442, 1, "M01", 9, "ap", -272442, 275139, 7, "M07", 12, "ap", 275139],
  ["roc", -273732, 4, "M04", 19, "broc", 273733, 273849, 9, "M09", 13, "roc", 273849],
];

for (const [calendar, minYear, minMonth, minMonthCode, minDay, minEra, minEraYear, maxYear, maxMonth, maxMonthCode, maxDay, maxEra, maxEraYear] of testData) {
  TemporalHelpers.assertPlainDateTime(min.withCalendar(calendar),
    minYear, minMonth, minMonthCode, minDay, 0, 0, 0, 0, 0, 1,
    `${calendar} minimum supported date`,
    minEra, minEraYear);
  TemporalHelpers.assertPlainDateTime(max.withCalendar(calendar),
    maxYear, maxMonth, maxMonthCode, maxDay, 23, 59, 59, 999, 999, 999,
    `${calendar} maximum supported date`,
    maxEra, maxEraYear);
}

{
  const calendar = "chinese";
  const minNonApproximated = new Temporal.PlainDateTime(1900, 1, 31);
  const maxNonApproximated = new Temporal.PlainDateTime(2101, 1, 28, 23, 59, 59, 999, 999, 999);
  TemporalHelpers.assertPlainDateTime(minNonApproximated.withCalendar(calendar),
    1900, 1, "M01", 1, 0, 0, 0, 0, 0, 0,
    `${calendar} minimum non-approximated date`);
  TemporalHelpers.assertPlainDateTime(maxNonApproximated.withCalendar(calendar),
    2100, 12, "M12", 29, 23, 59, 59, 999, 999, 999,
    `${calendar} maximum non-approximated date`);

  // Test that the min and max dates can be created, but don't care about the
  // conversion
  min.withCalendar(calendar);
  max.withCalendar(calendar);
}

{
  const calendar = "dangi";
  const minNonApproximated = new Temporal.PlainDateTime(1900, 1, 31);
  const maxNonApproximated = new Temporal.PlainDateTime(2051, 2, 10, 23, 59, 59, 999, 999, 999);
  TemporalHelpers.assertPlainDateTime(minNonApproximated.withCalendar(calendar),
    1900, 1, "M01", 1, 0, 0, 0, 0, 0, 0,
    `${calendar} minimum non-approximated date`);
  TemporalHelpers.assertPlainDateTime(maxNonApproximated.withCalendar(calendar),
    2050, 13, "M12", 29, 23, 59, 59, 999, 999, 999,
    `${calendar} maximum non-approximated date`);

  // Test that the min and max dates can be created, but don't care about the
  // conversion
  min.withCalendar(calendar);
  max.withCalendar(calendar);
}

// Additionally test the range of islamic-umalqura in which it does not fall
// back to islamic-civil
{
  const calendar = "islamic-umalqura";
  const minNonApproximated = new Temporal.PlainDateTime(1882, 11, 12);
  const maxNonApproximated = new Temporal.PlainDateTime(2077, 11, 16, 23, 59, 59, 999, 999, 999);
  TemporalHelpers.assertPlainDateTime(minNonApproximated.withCalendar(calendar),
    1300, 1, "M01", 1, 0, 0, 0, 0, 0, 0,
    `${calendar} minimum non-approximated date`,
    "ah", 1300);
  TemporalHelpers.assertPlainDateTime(maxNonApproximated.withCalendar(calendar),
    1500, 12, "M12", 30, 23, 59, 59, 999, 999, 999,
    `${calendar} maximum non-approximated date`,
    "ah", 1500);
}
