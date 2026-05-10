// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: A ZonedDateTime is valid in a property bag for a time zone
features: [Temporal]
---*/

const result = Temporal.ZonedDateTime.from({ year: 2000, month: 5, day: 2, timeZone: new Temporal.ZonedDateTime(0n, "UTC") });
assert.sameValue(result.timeZoneId, "UTC", 'Time zone created from ZonedDateTime object');
