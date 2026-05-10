// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withtimezone
description: The receiver's time zone is preserved in the return value
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1572342398_271_986_102n, "-07:00", "gregory");
const result = instance.withCalendar("japanese");
assert.sameValue(result.timeZoneId, "-07:00", "Time zone is preserved in return value");
