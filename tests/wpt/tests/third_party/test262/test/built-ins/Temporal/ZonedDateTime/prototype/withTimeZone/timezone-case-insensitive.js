// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withtimezone
description: Time zone names are case insensitive
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "UTC");

const timeZone = 'uTc';
const result = instance.withTimeZone(timeZone);
assert.sameValue(result.timeZoneId, 'UTC', `Time zone created from string "${timeZone}"`);
