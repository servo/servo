// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: A PlainDate object is copied, not returned directly
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const orig = new Temporal.PlainDate(2000, 5, 2);
const result = Temporal.PlainDate.from(orig);

TemporalHelpers.assertPlainDate(
  result,
  2000, 5, "M05", 2,
  "PlainDate is copied"
);

assert.sameValue(result.calendarId, orig.calendarId, "Calendar is copied");

assert.notSameValue(
  result,
  orig,
  "When a PlainDate is given, the returned value is not the original PlainDate"
);
