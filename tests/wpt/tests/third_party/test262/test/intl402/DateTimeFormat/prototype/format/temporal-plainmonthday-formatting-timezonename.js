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
const md = new Temporal.PlainMonthDay(1, 5, "gregory", 1972);

for (const timeZoneNameStyle of timeZoneNameStyles) {
  const dtf = new Intl.DateTimeFormat(locale, { timeZoneName: timeZoneNameStyle });

  const timeZoneDisplayName = dtf.formatToParts(Date.UTC(1972, 0, 5)).find(({ type }) => {
    return type === 'timeZoneName';
  }).value;

  const result = dtf.format(md);
  assert.sameValue(typeof result, "string",
    `can format a PlainMonthDay with timeZoneName = ${timeZoneNameStyle}`);
  assert.sameValue(result.indexOf(timeZoneDisplayName), -1,
    `"${result}" should not include ${timeZoneDisplayName}`);
}
