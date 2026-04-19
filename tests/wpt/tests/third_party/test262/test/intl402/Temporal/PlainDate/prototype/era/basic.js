// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.era
description: Basic tests for era property
features: [Temporal, Intl.Era-monthcode]
---*/

const instance = new Temporal.PlainDate(2000, 3, 6, "gregory");
assert.sameValue(instance.era, "ce");
