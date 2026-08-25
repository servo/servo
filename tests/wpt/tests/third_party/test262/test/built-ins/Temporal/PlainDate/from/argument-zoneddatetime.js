// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: A ZonedDateTime object is handled separately
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const calendar = "iso8601";
const zdt = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC", calendar);
const result = Temporal.PlainDate.from(zdt);

TemporalHelpers.assertPlainDate(
  result,
  2001, 9, "M09", 9,
  "ZonedDateTime is converted"
);

assert.sameValue(
  result.calendarId,
  calendar,
  "Calendar is copied"
);
