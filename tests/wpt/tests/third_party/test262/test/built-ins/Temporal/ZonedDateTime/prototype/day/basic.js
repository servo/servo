// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.day
description: The "day" property of Temporal.ZonedDateTime.prototype
features: [Temporal]
---*/

// (new Temporal.PlainDateTime(2021, 7, 15, 5, 30, 13)).toZonedDateTime("UTC")
assert.sameValue(new Temporal.ZonedDateTime(1626327013000000000n, "UTC").day, 15);
assert.sameValue(Temporal.ZonedDateTime.from('2019-03-18T05:30:13+00:00[UTC]').day, 18);
