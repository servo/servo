// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: from() gives sensible output at extremes of supported range
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

// Lunisolar/lunar calendars can't accurately predict celestial orbits for dates
// far into the past/future. Skip `chinese` and `dangi`. `islamic-umalqura` is
// okay because it is specified to fall back to `islamic-civil` outside the
// range of accuracy.

// Note that the earliest PlainYearMonth that can be constructed in a calendar
// is the earliest month whose first day occurs after ISO -271821-04-19

const testData = [
  ["buddhist", -271278, 5, "M05", "be", -271278, 1, 276303, 9, "M09", "be", 276303, 1],
  ["coptic", -272099, 4, "M04", "am", -272099, 27, 275471, 6, "M06", "am", 275471, 22],
  ["ethioaa", -266323, 4, "M04", "aa", -266323, 27, 281247, 6, "M06", "aa", 281247, 22],
  ["ethiopic", -271823, 4, "M04", "aa", -266323, 27, 275747, 6, "M06", "am", 275747, 22],
  ["gregory", -271821, 5, "M05", "bce", 271822, 1, 275760, 9, "M09", "ce", 275760, 1],
  ["hebrew", -268058, 12, "M12", "am", -268058, 16, 279517, 10, "M09", "am", 279517, 3],
  ["indian", -271899, 2, "M02", "shaka", -271899, 21, 275682, 7, "M07", "shaka", 275682, 23],
  ["islamic-civil", -280804, 4, "M04", "bh", 280805, 29, 283583, 6, "M06", "ah", 283583, 21],
  ["islamic-tbla", -280804, 4, "M04", "bh", 280805, 28, 283583, 6, "M06", "ah", 283583, 20],
  ["islamic-umalqura", -280804, 4, "M04", "bh", 280805, 29, 283583, 6, "M06", "ah", 283583, 21],
  ["japanese", -271821, 5, "M05", "bce", 271822, 1, 275760, 9, "M09", "reiwa", 273742, 1],
  ["persian", -272442, 2, "M02", "ap", -272442, 12, 275139, 7, "M07", "ap", 275139, 2],
  ["roc", -273732, 5, "M05", "broc", 273733, 1, 273849, 9, "M09", "roc", 273849, 1],
];

for (const [calendar, minYear, minMonth, minMonthCode, minEra, minEraYear, minISODay, maxYear, maxMonth, maxMonthCode, maxEra, maxEraYear, maxISODay] of testData) {
  const min = Temporal.PlainYearMonth.from({
    calendar,
    year: minYear,
    era: minEra,
    eraYear: minEraYear,
    month: minMonth,
    monthCode: minMonthCode,
  });
  TemporalHelpers.assertPlainYearMonth(min,
    minYear, minMonth, minMonthCode,
    `${calendar} minimum supported date`,
    minEra, minEraYear, minISODay);
  const max = Temporal.PlainYearMonth.from({
    calendar,
    year: maxYear,
    era: maxEra,
    eraYear: maxEraYear,
    month: maxMonth,
    monthCode: maxMonthCode,
  });
  TemporalHelpers.assertPlainYearMonth(max,
    maxYear, maxMonth, maxMonthCode,
    `${calendar} maximum supported date`,
    maxEra, maxEraYear, maxISODay);
}

{
  const calendar = "chinese";
  const minNonApproximated = Temporal.PlainYearMonth.from({ calendar, year: 1900, month: 1 });
  const maxNonApproximated = Temporal.PlainYearMonth.from({ calendar, year: 2100, month: 12 });
  TemporalHelpers.assertPlainYearMonth(minNonApproximated,
    1900, 1, "M01",
    `${calendar} minimum non-approximated date`,
    undefined, undefined, 31);
  TemporalHelpers.assertPlainYearMonth(maxNonApproximated,
    2100, 12, "M12", `${calendar} maximum non-approximated date`,
    undefined, undefined, 31);

  // Create dates far in the past and future but don't care about the conversion
  Temporal.PlainYearMonth.from({ calendar, year: -250000, month: 1 });
  Temporal.PlainYearMonth.from({ calendar, year: 250000, month: 1 });
}

{
  const calendar = "dangi";
  const minNonApproximated = Temporal.PlainYearMonth.from({ calendar, year: 1900, month: 1 });
  const maxNonApproximated = Temporal.PlainYearMonth.from({ calendar, year: 2050, month: 13 });
  TemporalHelpers.assertPlainYearMonth(minNonApproximated,
    1900, 1, "M01", `${calendar} minimum non-approximated date`,
    undefined, undefined, 31);
  TemporalHelpers.assertPlainYearMonth(maxNonApproximated,
    2050, 13, "M12", `${calendar} maximum non-approximated date`,
    undefined, undefined, 13);

  // Create dates far in the past and future but don't care about the conversion
  Temporal.PlainYearMonth.from({ calendar, year: -250000, month: 1 });
  Temporal.PlainYearMonth.from({ calendar, year: 250000, month: 1 });
}

// Additionally test the range of islamic-umalqura in which it does not fall
// back to islamic-civil
{
  const calendar = "islamic-umalqura";
  const minNonApproximated = Temporal.PlainYearMonth.from({ calendar, year: 1300, month: 1 });
  const maxNonApproximated = Temporal.PlainYearMonth.from({ calendar, year: 1500, month: 12 });
  TemporalHelpers.assertPlainYearMonth(minNonApproximated,
    1300, 1, "M01", `${calendar} minimum non-approximated date`,
    "ah", 1300, 12);
  TemporalHelpers.assertPlainYearMonth(maxNonApproximated,
    1500, 12, "M12", `${calendar} maximum non-approximated date`,
    "ah", 1500, 18);
}
