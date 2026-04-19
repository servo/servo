// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: from() gives sensible output at extremes of supported range
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

// Lunisolar/lunar calendars can't accurately predict celestial orbits for dates
// far into the past/future. Skip `chinese` and `dangi`. `islamic-umalqura` is
// okay because it is specified to fall back to `islamic-civil` outside the
// range of accuracy.

const testData = [
  ["buddhist", -271278, 4, "M04", 20, "be", -271278, 276303, 9, "M09", 13, "be", 276303],
  ["coptic", -272099, 3, "M03", 24, "am", -272099, 275471, 5, "M05", 22, "am", 275471],
  ["ethioaa", -266323, 3, "M03", 24, "aa", -266323, 281247, 5, "M05", 22, "aa", 281247],
  ["ethiopic", -271823, 3, "M03", 24, "aa", -266323, 275747, 5, "M05", 22, "am", 275747],
  ["gregory", -271821, 4, "M04", 20, "bce", 271822, 275760, 9, "M09", 13, "ce", 275760],
  ["hebrew", -268058, 11, "M11", 5, "am", -268058, 279517, 10, "M09", 11, "am", 279517],
  ["indian", -271899, 1, "M01", 30, "shaka", -271899, 275682, 6, "M06", 22, "shaka", 275682],
  ["islamic-civil", -280804, 3, "M03", 22, "bh", 280805, 283583, 5, "M05", 23, "ah", 283583],
  ["islamic-tbla", -280804, 3, "M03", 23, "bh", 280805, 283583, 5, "M05", 24, "ah", 283583],
  ["islamic-umalqura", -280804, 3, "M03", 22, "bh", 280805, 283583, 5, "M05", 23, "ah", 283583],
  ["japanese", -271821, 4, "M04", 20, "bce", 271822, 275760, 9, "M09", 13, "reiwa", 273742],
  ["persian", -272442, 1, "M01", 10, "ap", -272442, 275139, 7, "M07", 12, "ap", 275139],
  ["roc", -273732, 4, "M04", 20, "broc", 273733, 273849, 9, "M09", 13, "roc", 273849],
];

for (const [calendar, minYear, minMonth, minMonthCode, minDay, minEra, minEraYear, maxYear, maxMonth, maxMonthCode, maxDay, maxEra, maxEraYear] of testData) {
  const min = Temporal.ZonedDateTime.from({
    calendar,
    year: minYear,
    era: minEra,
    eraYear: minEraYear,
    month: minMonth,
    monthCode: minMonthCode,
    day: minDay,
    timeZone: "UTC",
  });
  assert.sameValue(min.epochNanoseconds, -86400_0000_0000_000_000_000n, `${calendar} minimum supported date`)

  const max = Temporal.ZonedDateTime.from({
    calendar,
    year: maxYear,
    era: maxEra,
    eraYear: maxEraYear,
    month: maxMonth,
    monthCode: maxMonthCode,
    day: maxDay,
    timeZone: "UTC",
  });
  assert.sameValue(max.epochNanoseconds, 86400_0000_0000_000_000_000n, `${calendar} maximum supported date`)
}

{
  const calendar = "chinese";
  const minNonApproximated = Temporal.ZonedDateTime.from({ calendar, year: 1900, month: 1, day: 1, timeZone: "UTC" });
  const maxNonApproximated = Temporal.ZonedDateTime.from({ calendar, year: 2100, month: 12, day: 29, hour: 23, minute: 59, second: 59, millisecond: 999, microsecond: 999, nanosecond: 999, timeZone: "UTC" });
  TemporalHelpers.assertPlainDateTime(minNonApproximated.toPlainDateTime(),
    1900, 1, "M01", 1, 0, 0, 0, 0, 0, 0,
    `${calendar} minimum non-approximated date`);
  TemporalHelpers.assertPlainDateTime(maxNonApproximated.toPlainDateTime(),
    2100, 12, "M12", 29, 23, 59, 59, 999, 999, 999,
    `${calendar} maximum non-approximated date`);

  // Create dates far in the past and future but don't care about the conversion
  Temporal.ZonedDateTime.from({ calendar, year: -250000, month: 1, day: 1, timeZone: "UTC" });
  Temporal.ZonedDateTime.from({ calendar, year: 250000, month: 1, day: 1, timeZone: "UTC" });
}

{
  const calendar = "dangi";
  const minNonApproximated = Temporal.ZonedDateTime.from({ calendar, year: 1900, month: 1, day: 1, timeZone: "UTC" });
  const maxNonApproximated = Temporal.ZonedDateTime.from({ calendar, year: 2050, month: 13, day: 29, hour: 23, minute: 59, second: 59, millisecond: 999, microsecond: 999, nanosecond: 999, timeZone: "UTC" });
  TemporalHelpers.assertPlainDateTime(minNonApproximated.toPlainDateTime(),
    1900, 1, "M01", 1, 0, 0, 0, 0, 0, 0, `${calendar} minimum non-approximated date`);
  TemporalHelpers.assertPlainDateTime(maxNonApproximated.toPlainDateTime(),
    2050, 13, "M12", 29, 23, 59, 59, 999, 999, 999, `${calendar} maximum non-approximated date`);

  // Create dates far in the past and future but don't care about the conversion
  Temporal.ZonedDateTime.from({ calendar, year: -250000, month: 1, day: 1, timeZone: "UTC" });
  Temporal.ZonedDateTime.from({ calendar, year: 250000, month: 1, day: 1, timeZone: "UTC" });
}

// Additionally test the range of islamic-umalqura in which it does not fall
// back to islamic-civil
{
  const calendar = "islamic-umalqura";
  const minNonApproximated = Temporal.ZonedDateTime.from({ calendar, year: 1300, month: 1, day: 1, timeZone: "UTC" });
  const maxNonApproximated = Temporal.ZonedDateTime.from({ calendar, year: 1500, month: 12, day: 30, hour: 23, minute: 59, second: 59, millisecond: 999, microsecond: 999, nanosecond: 999, timeZone: "UTC" });
  TemporalHelpers.assertPlainDateTime(minNonApproximated.toPlainDateTime(),
    1300, 1, "M01", 1, 0, 0, 0, 0, 0, 0, `${calendar} minimum non-approximated date`,
    "ah", 1300);
  TemporalHelpers.assertPlainDateTime(maxNonApproximated.toPlainDateTime(),
    1500, 12, "M12", 30, 23, 59, 59, 999, 999, 999, `${calendar} maximum non-approximated date`,
    "ah", 1500);
}
