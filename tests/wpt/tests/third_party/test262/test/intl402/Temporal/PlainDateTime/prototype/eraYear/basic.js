// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.erayear
description: Basic tests for eraYear property
features: [Temporal, Intl.Era-monthcode]
---*/

const instance = new Temporal.PlainDateTime(2000, 3, 6, 12, 34, 56, 234, 563, 734, "gregory");
assert.sameValue(instance.eraYear, 2000);
