// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.microsecond
description: Basic functionality
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(3721_001_001_001n, "UTC");
assert.sameValue(instance.microsecond, 1);
