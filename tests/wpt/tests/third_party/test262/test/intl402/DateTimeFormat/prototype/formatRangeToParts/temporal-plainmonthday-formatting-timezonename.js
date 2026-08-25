// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: >
  PlainMonthDay can be formatted with a formatter created with the timeZoneName
  option, but no time zone name is included.
locale: [en-US-u-ca-gregory]
features: [Temporal]
---*/

const locale = "en-US-u-ca-gregory";
const timeZoneNameStyles = [
  "long", "short", "shortOffset", "longOffset", "shortGeneric", "longGeneric"
];
const md1 = new Temporal.PlainMonthDay(1, 5, "gregory", 1972);
const md2 = new Temporal.PlainMonthDay(1, 6, "gregory", 1972);

for (const timeZoneNameStyle of timeZoneNameStyles) {
  const dtf = new Intl.DateTimeFormat(locale, { timeZoneName: timeZoneNameStyle });
  const result = dtf.formatRangeToParts(md1, md2);
  assert(Array.isArray(result),
    `can format a PlainMonthDay with timeZoneName = ${timeZoneNameStyle}`);
  for (const { type } of result) {
    assert.notSameValue(type, "timeZoneName",
      `formatting a PlainMonthDay with timeZoneName = ${timeZoneNameStyle} should not print a time zone`);
  }
}
