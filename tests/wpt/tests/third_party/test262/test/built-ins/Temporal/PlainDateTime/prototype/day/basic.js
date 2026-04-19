// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.day
description: The "day" property of Temporal.PlainDateTime.prototype
features: [Temporal]
---*/

assert.sameValue((new Temporal.PlainDateTime(2021, 7, 15, 5, 30, 13)).day, 15);
assert.sameValue(Temporal.PlainDateTime.from('2019-03-18T05:30:13').day, 18);
