// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: A ZonedDateTime object is copied, not returned directly
features: [Temporal]
---*/

const orig = new Temporal.ZonedDateTime(946684800_000_000_010n, "UTC");
const result = Temporal.ZonedDateTime.from(orig);

assert.sameValue(result.epochNanoseconds, 946684800_000_000_010n, "ZonedDateTime is copied");
assert.sameValue(result.timeZoneId, orig.timeZoneId, "time zone is the same");
assert.sameValue(result.calendarId, orig.calendarId, "calendar is the same");

assert.notSameValue(
  result,
  orig,
  "When a ZonedDateTime is given, the returned value is not the original ZonedDateTime"
);
