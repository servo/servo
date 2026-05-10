// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: from() throws for month before earlier supported month, and month after latest supported month
features: [Temporal, Intl.Era-monthcode]
---*/

// Lunisolar/lunar calendars can't accurately predict celestial orbits for dates
// far into the past/future. Skip `chinese` and `dangi`. `islamic-umalqura` is
// okay because it is specified to fall back to `islamic-civil` outside the
// range of accuracy.

// Note that the earliest PlainYearMonth that can be constructed in a calendar
// is the earliest month whose first day occurs after ISO -271821-04-19

const testData = [
  ["buddhist", -271278, 3, "M03", "be", -271278, 276303, 10, "M10", "be", 276303],
  ["coptic", -272099, 3, "M03", "am", -272099, 275471, 7, "M07", "am", 275471],
  ["ethioaa", -266323, 3, "M03", "aa", -266323, 281247, 7, "M07", "aa", 281247],
  ["ethiopic", -271823, 3, "M03", "aa", -266323, 275747, 7, "M07", "am", 275747],
  ["gregory", -271821, 3, "M03", "bce", 271822, 275760, 10, "M10", "ce", 275760],
  ["hebrew", -268058, 10, "M10", "am", -268058, 279517, 11, "M11", "am", 279517],
  ["indian", -271899, 1, "M01", "shaka", -271899, 275682, 8, "M08", "shaka", 275682],
  ["islamic-civil", -280804, 3, "M03", "bh", 280805, 283583, 7, "M07", "ah", 283583],
  ["islamic-tbla", -280804, 3, "M03", "bh", 280805, 283583, 7, "M07", "ah", 283583],
  ["islamic-umalqura", -280804, 3, "M03", "bh", 280805, 283583, 7, "M07", "ah", 283583],
  ["japanese", -271821, 3, "M03", "bce", 271822, 275760, 10, "M10", "reiwa", 273742],
  ["persian", -272443, 12, "M12", "ap", -272443, 275139, 8, "M08", "ap", 275139],
  ["roc", -273732, 3, "M03", "broc", 273733, 273849, 10, "M10", "roc", 273849],
];

for (const [calendar, minYear, minMonth, minMonthCode, minEra, minEraYear, maxYear, maxMonth, maxMonthCode, maxEra, maxEraYear] of testData) {
  assert.throws(RangeError, () => Temporal.PlainYearMonth.from({
    calendar,
    year: minYear,
    era: minEra,
    eraYear: minEraYear,
    month: minMonth,
    monthCode: minMonthCode,
  }), `calendar ${calendar}: month before min month should throw`);

  assert.throws(RangeError, () => Temporal.PlainYearMonth.from({
    calendar,
    year: maxYear,
    era: maxEra,
    eraYear: maxEraYear,
    month: maxMonth,
    monthCode: maxMonthCode,
  }), `calendar ${calendar}: month after max month should throw`);
}
