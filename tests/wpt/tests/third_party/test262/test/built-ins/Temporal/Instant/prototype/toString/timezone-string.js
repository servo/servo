// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Time zone IDs are valid input for a time zone
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);

const result1 = instance.toString({ timeZone: "UTC" });
assert.sameValue(result1.slice(-6), "+00:00", "Time zone created from string 'UTC'");

const result2 = instance.toString({ timeZone: "-01:30" });
assert.sameValue(result2.slice(-6), "-01:30", "Time zone created from string '-01:30'");
