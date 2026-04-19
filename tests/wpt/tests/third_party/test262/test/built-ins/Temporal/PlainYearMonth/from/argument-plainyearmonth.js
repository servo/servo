// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: A PlainYearMonth object is copied, not returned directly
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const orig = new Temporal.PlainYearMonth(2000, 5, undefined, 7);
const result = Temporal.PlainYearMonth.from(orig);

TemporalHelpers.assertPlainYearMonth(
  result,
  2000, 5, "M05",
  "PlainYearMonth is copied",
  /* era = */ undefined, /* eraYear = */ undefined, /* isoDay = */ 7
);

assert.sameValue(result.calendarId, orig.calendarId, "Calendar is copied");

assert.notSameValue(
  result,
  orig,
  "When a PlainYearMonth is given, the returned value is not the original PlainYearMonth"
);
