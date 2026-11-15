// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: DateTimeFormat or Locale rejected in calendar property
info: |
  Non-Temporal objects with a [[Calendar]] internal slot; see
  https://github.com/tc39/ecma262/pull/3966/changes#paneldiscussion_r4284855527
features: [Temporal, Intl.Locale]
---*/

const intlCalendarObjects = [
  ["Intl.DateTimeFormat", new Intl.DateTimeFormat("en", { calendar: "iso8601" })],
  ["Intl.Locale", new Intl.Locale("en", { calendar: "iso8601" })],
];

for (const [name, calendar] of intlCalendarObjects) {
  const bag = { year: 1976, monthCode: "M11", day: 18, timeZone: "UTC", calendar };
  assert.throws(
    TypeError,
    () => Temporal.ZonedDateTime.from(bag),
    `${name} instance is not accepted as a calendar`
  );
}
