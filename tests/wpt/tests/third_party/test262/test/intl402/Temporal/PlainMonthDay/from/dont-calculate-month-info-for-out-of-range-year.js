// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: >
  PlainMonthDay.from bails out early without constraining the month, when the
  year is out of range.
features: [Temporal, Intl.Era-monthcode]
---*/

const testData = [
  ["buddhist", "M02", 29, "be"],
  ["chinese", "M06L", 30],
  ["coptic", "M13", 6, "am"],
  ["dangi", "M06L", 30],
  ["ethioaa", "M13", 6, "aa"],
  ["ethiopic", "M13", 6, "aa"],
  ["gregory", "M02", 29, "ce", "bce"],
  ["hebrew", "M05L", 29, "am"],
  ["indian", "M01", 31, "shaka"],
  ["islamic-civil", "M12", 30, "ah", "bh"],
  ["islamic-tbla", "M12", 30, "ah", "bh"],
  ["islamic-umalqura", "M12", 30, "ah", "bh"],
  ["japanese", "M02", 29, "reiwa", "bce"],
  ["persian", "M12", 30, "ap"],
  ["roc", "M02", 29, "roc", "broc"],
];

for (const [calendar, monthCode, day, posEra = undefined, negEra = undefined] of testData) {
  assert.throws(RangeError, function () {
    Temporal.PlainMonthDay.from({ year: -999999, monthCode, day, calendar });
  }, `${calendar} bails out when year is -999999`);

  assert.throws(RangeError, function () {
    Temporal.PlainMonthDay.from({ year: 999999, monthCode, day, calendar });
  }, `${calendar} bails out when year is +999999`);

  if (posEra) {
    assert.throws(RangeError, function () {
      Temporal.PlainMonthDay.from({ eraYear: 999999, era: posEra, monthCode, day, calendar });
    }, `${calendar} bails out when era year is +999999 ${posEra}`);

    if (negEra) {
      assert.throws(RangeError, function () {
        Temporal.PlainMonthDay.from({ eraYear: 999999, era: negEra, monthCode, day, calendar });
      }, `${calendar} bails out when era year is +999999 ${negEra}`);
    } else {
      assert.throws(RangeError, function () {
        Temporal.PlainMonthDay.from({ eraYear: -999999, era: posEra, monthCode, day, calendar });
      }, `${calendar} bails out when era year is -999999 ${posEra}`);
    }
  }
}
