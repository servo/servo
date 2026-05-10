// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withcalendar
description: Objects of a subclass are never created as return values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkSubclassingIgnored(
  Temporal.ZonedDateTime,
  [10n, "UTC"],
  "withCalendar",
  ["iso8601"],
  (result) => {
    assert.sameValue(result.epochNanoseconds, 10n, "epochNanoseconds result");
    assert.sameValue(result.year, 1970, "year result");
    assert.sameValue(result.month, 1, "month result");
    assert.sameValue(result.day, 1, "day result");
    assert.sameValue(result.hour, 0, "hour result");
    assert.sameValue(result.minute, 0, "minute result");
    assert.sameValue(result.second, 0, "second result");
    assert.sameValue(result.millisecond, 0, "millisecond result");
    assert.sameValue(result.microsecond, 0, "microsecond result");
    assert.sameValue(result.nanosecond, 10, "nanosecond result");
    assert.sameValue(result.calendarId, "iso8601", "calendar result");
  },
);
