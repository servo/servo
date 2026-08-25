// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatet.prototype.era
description: Basic tests for era property
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 3, 6);
assert.sameValue(instance.era, undefined);
