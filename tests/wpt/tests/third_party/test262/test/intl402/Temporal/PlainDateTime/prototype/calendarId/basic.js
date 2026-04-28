// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.calendarid
description: Basic functionality of calendarId property
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 3, 6, 12, 34, 58, 233, 412, 776, "gregory");
assert.sameValue(instance.calendarId, "gregory");
