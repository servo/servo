// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: A PlainMonthDay object is copied, not returned directly
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const orig = new Temporal.PlainMonthDay(5, 2, undefined, 2000);
const result = Temporal.PlainMonthDay.from(orig);

TemporalHelpers.assertPlainMonthDay(
  result,
  "M05", 2,
  "PlainMonthDay is copied",
  /* isoYear = */ 2000
);

assert.sameValue(result.calendarId, orig.calendarId, "Calendar is copied");

assert.notSameValue(
  result,
  orig,
  "When a PlainMonthDay is given, the returned value is not the original PlainMonthDay"
);
