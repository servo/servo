// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Instances with different calendars are not equal to each other
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "-05:00", "iso8601");
const instance2 = new Temporal.ZonedDateTime(0n, "-05:00", "gregory");
assert(!instance.equals(instance2), "Instances with different calendars are not equal");
