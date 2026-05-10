// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: >
  PlainYearMonth can be formatted with a formatter created with the timeZoneName
  option, but no time zone name is included.
locale: [en-US]
features: [Temporal]
---*/

const locale = "en-US";
const timeZoneNameStyles = [
  "long", "short", "shortOffset", "longOffset", "shortGeneric", "longGeneric"
];
const ym1 = new Temporal.PlainYearMonth(2026, 1, "gregory", 1);
const ym2 = new Temporal.PlainYearMonth(2026, 2, "gregory", 1);

for (const timeZoneNameStyle of timeZoneNameStyles) {
  const dtf = new Intl.DateTimeFormat(locale, { timeZoneName: timeZoneNameStyle });
  const result = dtf.formatRangeToParts(ym1, ym2);
  assert(Array.isArray(result),
    `can format a PlainYearMonth with timeZoneName = ${timeZoneNameStyle}`);
  for (const { type } of result) {
    assert.notSameValue(type, "timeZoneName",
      `formatting a PlainYearMonth with timeZoneName = ${timeZoneNameStyle} should not print a time zone`);
  }
}
