// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.from
description: Time zone names are case insensitive
features: [Temporal]
---*/

const timeZone = 'uTc';
const result = Temporal.ZonedDateTime.from({ year: 2000, month: 5, day: 2, timeZone });
assert.sameValue(result.timeZoneId, 'UTC', `Time zone created from string "${timeZone}"`);
