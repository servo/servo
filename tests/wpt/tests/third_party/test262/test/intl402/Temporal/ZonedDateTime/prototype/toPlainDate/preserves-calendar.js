// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toplaindate
description: The receiver's calendar is preserved in the return value
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1572342398_271_986_102n, "-07:00", "gregory");
const result = instance.toPlainDate();
assert.sameValue(result.calendarId, "gregory", "Calendar is preserved in return value");
