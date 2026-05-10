// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat.prototype.formattoparts
description: >
  Verifies that DateTimeFormat formats dates in lunisolar calendars with leap
  leap months (Chinese, Dangi, Hebrew)
locale: [en-u-ca-chinese, en-u-ca-dangi, en-u-ca-hebrew]
features: [Intl.Era-monthcode]
---*/

const tests = [
  ["chinese", 2020, 4, 23, "relatedYear"],  // May 23, 2020 is in the leap month M04L
  ["chinese", 2019, 4, 15, "relatedYear"],  // In regular month M04L
  ["dangi", 2020, 4, 23, "relatedYear"],    // As above
  ["dangi", 2019, 4, 15, "relatedYear"],
  ["hebrew", 2024, 2, 15, "year"],          // In M05L (Adar I) of leap year 5784
  ["hebrew", 2023, 2, 15, "year"],          // In M06 (Adar) of common year 5783
];

for (const [calendar, isoYear, zeroMonth, day, yearPartName] of tests) {
  const formatter = new Intl.DateTimeFormat(`en-u-ca-${calendar}`, {
    year: "numeric",
    month: "long",
    day: "numeric"
  });

  const date = new Date(isoYear, zeroMonth, day);
  const parts = formatter.formatToParts(date);

  const monthPart = parts.find(({ type }) => type === "month");
  assert.notSameValue(monthPart, undefined, `${calendar} calendar date should have month part`);
  assert.sameValue(typeof monthPart.value, "string", `${calendar} month part value should be a string`);

  const yearPart = parts.find(({ type }) => type === yearPartName);
  assert.notSameValue(yearPart, undefined, `${calendar} calendar date should have year part`);

  const formatted = formatter.format(date);
  const reconstructed = parts.map((part) => part.value).join("");
  assert.sameValue(formatted, reconstructed,
    `format() and formatToParts() should produce consistent results for ${calendar} calendar`);
}
