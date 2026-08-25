// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Not-yet-adopted calendar IDs are rejected
includes: [temporalHelpers.js]
features: [Intl.Era-monthcode, Temporal]
---*/

const okDate = new Temporal.ZonedDateTime(0n, "UTC");

for (const calendar of TemporalHelpers.NotYetSupportedCalendars) {
  assert.throws(RangeError, function () {
    Temporal.ZonedDateTime.compare(`1970-01-01[UTC][u-ca=${calendar}]`, okDate);
  }, `${calendar} is not yet supported (first argument, string)`);
  assert.throws(RangeError, function () {
    Temporal.ZonedDateTime.compare({ year: 1970, month: 1, day: 1, timeZone: "UTC", calendar }, okDate);
  }, `${calendar} is not yet supported (first argument, property bag)`);
  assert.throws(RangeError, function () {
    Temporal.ZonedDateTime.compare(okDate, `1970-01-01[UTC][u-ca=${calendar}]`);
  }, `${calendar} is not yet supported (second argument, string)`);
  assert.throws(RangeError, function () {
    Temporal.ZonedDateTime.compare(okDate, { year: 1970, month: 1, day: 1, timeZone: "UTC", calendar });
  }, `${calendar} is not yet supported (second argument, property bag)`);
}
