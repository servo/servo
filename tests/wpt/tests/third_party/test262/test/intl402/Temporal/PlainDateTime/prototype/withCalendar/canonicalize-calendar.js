// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withcalendar
description: Calendar ID is canonicalized
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2024, 7, 2, 12, 34);
const result = instance.withCalendar("islamicc");
assert.sameValue(result.calendarId, "islamic-civil", "calendar ID is canonicalized");
