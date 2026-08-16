// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Not-yet-adopted calendar IDs are rejected
includes: [temporalHelpers.js]
features: [Intl.Era-monthcode, Temporal]
---*/

for (const calendar of TemporalHelpers.NotYetSupportedCalendars) {
  assert.throws(RangeError, function () {
    Temporal.ZonedDateTime.from(`1970-01-01[UTC][u-ca=${calendar}]`);
  }, `${calendar} is not yet supported (string)`);
  assert.throws(RangeError, function () {
    Temporal.ZonedDateTime.from({ year: 1970, month: 1, day: 1, timeZone: "UTC", calendar });
  }, `${calendar} is not yet supported (property bag)`);
}
