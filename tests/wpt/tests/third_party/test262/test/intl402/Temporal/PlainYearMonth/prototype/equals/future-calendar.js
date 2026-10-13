// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.equals
description: Not-yet-adopted calendar IDs are rejected
includes: [temporalHelpers.js]
features: [Intl.Era-monthcode, Temporal]
---*/

const okDate = new Temporal.PlainYearMonth(1970, 1);

for (const calendar of TemporalHelpers.NotYetSupportedCalendars) {
  assert.throws(RangeError, function () {
    okDate.equals(`1970-01-01[u-ca=${calendar}]`);
  }, `${calendar} is not yet supported (string)`);
  assert.throws(RangeError, function () {
    okDate.equals({ year: 1970, month: 1, calendar });
  }, `${calendar} is not yet supported (property bag)`);
}
