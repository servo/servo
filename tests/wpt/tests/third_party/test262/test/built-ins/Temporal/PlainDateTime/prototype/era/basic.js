// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.era
description: Basic tests for era property
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 3, 6, 12, 34, 56, 567, 234, 123);
assert.sameValue(instance.era, undefined);
