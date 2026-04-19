// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: >
  PlainTime can be formatted with a formatter created with the timeZoneName
  option, but no time zone name is included.
locale: [en-US]
features: [Temporal]
---*/

const locale = "en-US";
const timeZoneNameStyles = [
  "long", "short", "shortOffset", "longOffset", "shortGeneric", "longGeneric"
];
const time = new Temporal.PlainTime(12, 34);

for (const timeZoneNameStyle of timeZoneNameStyles) {
  const dtf = new Intl.DateTimeFormat(locale, { timeZoneName: timeZoneNameStyle });

  const timeZoneDisplayName = dtf.formatToParts(Date.UTC(1970, 0, 1, 12, 34)).find(({ type }) => {
    return type === 'timeZoneName';
  }).value;

  const result = dtf.format(time);
  assert.sameValue(typeof result, "string",
    `can format a PlainTime with timeZoneName = ${timeZoneNameStyle}`);
  assert.sameValue(result.indexOf(timeZoneDisplayName), -1,
    `"${result}" should not include ${timeZoneDisplayName}`);
}
