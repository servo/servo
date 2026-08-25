// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.calendarid
description: Basic functionality of calendarId property
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "Europe/Madrid", "gregory");
assert.sameValue(instance.calendarId, "gregory");
