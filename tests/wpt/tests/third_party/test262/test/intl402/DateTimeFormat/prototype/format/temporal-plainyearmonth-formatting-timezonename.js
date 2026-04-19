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
const ym = new Temporal.PlainYearMonth(2026, 1, "gregory", 1);

for (const timeZoneNameStyle of timeZoneNameStyles) {
  const dtf = new Intl.DateTimeFormat(locale, { timeZoneName: timeZoneNameStyle });

  const timeZoneDisplayName = dtf.formatToParts(Date.UTC(2026, 0, 1)).find(({ type }) => {
    return type === 'timeZoneName';
  }).value;

  const result = dtf.format(ym);
  assert.sameValue(typeof result, "string",
    `can format a PlainYearMonth with timeZoneName = ${timeZoneNameStyle}`);
  assert.sameValue(result.indexOf(timeZoneDisplayName), -1,
    `"${result}" should not include ${timeZoneDisplayName}`);
}
