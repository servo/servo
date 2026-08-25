// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.second
description: Basic functionality
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2026, 3, 6, 12, 34, 56, 987, 654, 321);
assert.sameValue(instance.second, 56);
