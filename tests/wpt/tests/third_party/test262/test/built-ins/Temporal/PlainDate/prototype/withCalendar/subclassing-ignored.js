// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.withcalendar
description: Objects of a subclass are never created as return values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkSubclassingIgnored(
  Temporal.PlainDate,
  [2000, 5, 2],
  "withCalendar",
  ["iso8601"],
  (result) => {
    TemporalHelpers.assertPlainDate(result, 2000, 5, "M05", 2);
    assert.sameValue(result.calendarId, "iso8601", "calendar result");
  },
);
