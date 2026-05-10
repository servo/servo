// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Calendar era code is canonicalized (non-Gregorian calendars)
features: [Temporal, Intl.Era-monthcode]
---*/


const calendarEraAliases = [
  { calendar: "japanese", canonicalizedEra: "ce", alias: "ad" },
  { calendar: "japanese", canonicalizedEra: "bce", alias: "bc" }
];


for (const calendarEraAlias of calendarEraAliases) {
  const calendar = Temporal.PlainDateTime.from({
    calendar: calendarEraAlias.calendar,
    era: calendarEraAlias.alias,
    eraYear: 1,
    month: 1,
    day: 1
  });
  assert.sameValue(calendar.era, calendarEraAlias.canonicalizedEra, calendar.era + " should canonicalize to " + calendarEraAlias.canonicalizedEra)
}
