// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.zoneddatetimeiso
description: Time zone names are case insensitive
features: [Temporal]
---*/

const timeZone = 'UtC';
const result = Temporal.Now.zonedDateTimeISO(timeZone);
assert.sameValue(result.timeZoneId, 'UTC', `Time zone created from string "${timeZone}"`);
