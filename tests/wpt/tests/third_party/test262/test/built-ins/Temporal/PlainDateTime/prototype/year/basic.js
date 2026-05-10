// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.year
description: The "year" property of Temporal.PlainDateTime.prototype
features: [Temporal]
---*/

assert.sameValue((new Temporal.PlainDateTime(2021, 7, 15, 15, 30, 26, 123, 456, 789)).year, 2021);
assert.sameValue(Temporal.PlainDateTime.from('2019-03-15T15:30:26').year, 2019);
