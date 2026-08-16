// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Not-yet-adopted calendar IDs are rejected
includes: [temporalHelpers.js]
features: [Intl.Era-monthcode, Temporal]
---*/

for (const calendar of TemporalHelpers.NotYetSupportedCalendars) {
  assert.throws(RangeError, function () {
    Temporal.PlainDateTime.from(`1970-01-01[u-ca=${calendar}]`);
  }, `${calendar} is not yet supported (string)`);
  assert.throws(RangeError, function () {
    Temporal.PlainDateTime.from({ year: 1970, month: 1, day: 1, calendar });
  }, `${calendar} is not yet supported (property bag)`);
}
