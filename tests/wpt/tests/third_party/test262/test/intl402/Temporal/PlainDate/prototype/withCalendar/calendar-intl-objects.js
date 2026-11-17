// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.withcalendar
description: DateTimeFormat and Locale rejected as calendar identifiers
info: |
  Non-Temporal objects with a [[Calendar]] internal slot; see
  https://github.com/tc39/ecma262/pull/3966/changes#paneldiscussion_r4284855527
features: [Temporal, Intl.Locale]
---*/

const intlCalendarObjects = [
  ["Intl.DateTimeFormat", new Intl.DateTimeFormat("en", { calendar: "iso8601" })],
  ["Intl.Locale", new Intl.Locale("en", { calendar: "iso8601" })],
];

const instance = new Temporal.PlainDate(1976, 11, 18);

for (const [name, calendarLike] of intlCalendarObjects) {
  assert.throws(
    TypeError,
    () => instance.withCalendar(calendarLike),
    `${name} instance is not accepted as a calendar`
  );
}
