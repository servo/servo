// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sup-temporal.plainmonthday.prototype.tolocalestring
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
  "month",
  "day",
];
const calendar = new Intl.DateTimeFormat("en").resolvedOptions().calendar;
const md = new Temporal.PlainMonthDay(4, 17, calendar);

// dateStyle does not conflict with PlainMonthDay
md.toLocaleString("en", { dateStyle: "short" });

assert.throws(TypeError, function () {
  md.toLocaleString("en", { timeStyle: "short" });
}, "timeStyle conflicts with PlainMonthDay");

for (const option of conflictingOptions) {
  assert.throws(TypeError, function() {
    md.toLocaleString("en", { [option]: "numeric", dateStyle: "short" });
  }, `${option} conflicts with dateStyle`);

  // dateStyle or timeStyle present but undefined does not conflict
  md.toLocaleString("en", { [option]: "numeric", dateStyle: undefined });
  md.toLocaleString("en", { [option]: "numeric", timeStyle: undefined });
}
