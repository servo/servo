// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
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
  const bag = { year: 2019, monthCode: "M11", calendar };
  assert.throws(
    TypeError,
    () => Temporal.PlainYearMonth.from(bag),
    `${name} instance is not accepted as a calendar`
  );
}
