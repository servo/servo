// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Properly constrain a day that is one past a leap day
features: [Temporal]
---*/

const tests = [
  ["buddhist", "M02", 30],
  ["chinese", "M01", 31],
  ["coptic", "M13", 7],
  ["dangi", "M01", 31],
  ["ethioaa", "M13", 7],
  ["ethiopic", "M13", 7],
  ["gregory", "M02", 30],
  ["hebrew", "M02", 31],
  ["indian", "M01", 32],
  ["islamic-civil", "M01", 31],
  ["islamic-tbla", "M01", 31],
  ["islamic-umalqura", "M01", 31],
  ["japanese", "M02", 30],
  ["persian", "M12", 31],
  ["roc", "M02", 30],
];

for (const [calendar, monthCode, day] of tests) {
  const md = Temporal.PlainMonthDay.from({ calendar, monthCode, day }, { overflow: "constrain" });
  assert.sameValue(md.day, day - 1,
    `${calendar}: ${monthCode}-${day} should constrain to ${day - 1}, not ${day - 2}`)
}
