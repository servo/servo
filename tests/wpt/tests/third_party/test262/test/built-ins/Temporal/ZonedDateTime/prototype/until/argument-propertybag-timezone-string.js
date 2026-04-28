// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Time zone IDs are valid input for a time zone
features: [Temporal]
---*/

const instance1 = new Temporal.ZonedDateTime(0n, "UTC");
assert(instance1.until({ year: 1970, month: 1, day: 1, timeZone: "UTC" }).blank, "Time zone created from string 'UTC'");

const instance2 = new Temporal.ZonedDateTime(0n, "-01:30");
assert(instance2.until({ year: 1969, month: 12, day: 31, hour: 22, minute: 30, timeZone: "-01:30" }).blank, "Time zone created from string '-01:30'");
