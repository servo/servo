// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.dayofweek
description: Basic functionality
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(217178610_123_456_789n, "UTC");
assert.sameValue(instance.dayOfWeek, 4);
