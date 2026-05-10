// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: A PlainDateTime object is copied, not returned directly
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const orig = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 20, 123, 456, 789);
const result = Temporal.PlainDateTime.from(orig);

TemporalHelpers.assertPlainDateTime(
  result,
  1976, 11, "M11", 18, 15, 23, 20, 123, 456, 789,
  "PlainDateTime is copied"
);

assert.sameValue(result.calendarId, orig.calendarId, "Calendar is copied");

assert.notSameValue(
  result,
  orig,
  "When a PlainDateTime is given, the returned value is not the original PlainDateTime"
);
