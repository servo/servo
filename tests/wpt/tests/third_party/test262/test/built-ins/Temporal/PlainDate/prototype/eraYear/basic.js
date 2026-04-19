// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.erayear
description: Basic tests for eraYear property
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 3, 6);
assert.sameValue(instance.eraYear, undefined);
