// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tozoneddatetimeiso
description: >
  toZonedDateTimeISO() results in a ZonedDateTime with builtin ISO calendar
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);
const result = instance.toZonedDateTimeISO("UTC");
assert.sameValue(result.calendarId, "iso8601", "calendar string is iso8601");
