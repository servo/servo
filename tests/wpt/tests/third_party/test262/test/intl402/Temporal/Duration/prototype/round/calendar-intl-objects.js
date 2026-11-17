// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: DateTimeFormat or Locale rejected in relativeTo.calendar option
info: |
  Non-Temporal objects with a [[Calendar]] internal slot; see
  https://github.com/tc39/ecma262/pull/3966/changes#paneldiscussion_r4284855527
features: [Temporal, Intl.Locale]
---*/

const intlCalendarObjects = [
  ["Intl.DateTimeFormat", new Intl.DateTimeFormat("en", { calendar: "iso8601" })],
  ["Intl.Locale", new Intl.Locale("en", { calendar: "iso8601" })],
];

const duration = new Temporal.Duration(1, 1, 1, 1);

for (const [name, calendar] of intlCalendarObjects) {
  const relativeTo = { year: 2000, monthCode: "M01", day: 1, calendar };
  assert.throws(
    TypeError,
    () => duration.round({ smallestUnit: "months", relativeTo }),
    `${name} instance is not accepted as a calendar`
  );
}
