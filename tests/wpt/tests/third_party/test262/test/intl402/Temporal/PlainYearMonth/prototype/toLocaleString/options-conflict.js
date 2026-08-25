// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sup-temporal.plainyearmonth.prototype.tolocalestring
description: >
    Conflicting properties of dateStyle must be rejected with a TypeError for the options argument
info: |
  CreateDateTimeFormat:

  45. If _dateStyle_ is not *undefined* or _timeStyle_ is not *undefined*, then
    a. If _hasExplicitFormatComponents_ is *true*, then
      i. Throw a *TypeError* exception.
    b. If _required_ is ~date~ and _timeStyle_ is not *undefined*, then
      i. Throw a *TypeError* exception.
features: [Temporal]
---*/

const conflictingOptions = [
  ["era", "short"],
  ["year", "numeric"],
  ["month", "numeric"],
];
const calendar = new Intl.DateTimeFormat("en").resolvedOptions().calendar;
const ym = new Temporal.PlainYearMonth(2024, 4, calendar);

// dateStyle does not conflict with PlainYearMonth
ym.toLocaleString("en", { dateStyle: "short" });

assert.throws(TypeError, function () {
  ym.toLocaleString("en", { timeStyle: "short" });
}, "timeStyle conflicts with PlainYearMonth");

for (const [option, value] of conflictingOptions) {
  assert.throws(TypeError, function() {
    ym.toLocaleString("en", { [option]: value, dateStyle: "short" });
  }, `${option} conflicts with dateStyle`);

  // dateStyle or timeStyle present but undefined does not conflict
  ym.toLocaleString("en", { [option]: value, dateStyle: undefined });
  ym.toLocaleString("en", { [option]: value, timeStyle: undefined });
}
