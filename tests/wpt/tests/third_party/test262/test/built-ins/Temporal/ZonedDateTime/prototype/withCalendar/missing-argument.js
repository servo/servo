// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withcalendar
description: TypeError thrown when calendar argument not given
features: [Temporal]
---*/

const zonedDateTime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");
assert.throws(TypeError, () => zonedDateTime.withCalendar(), "missing argument");
assert.throws(TypeError, () => zonedDateTime.withCalendar(undefined), "undefined argument");
