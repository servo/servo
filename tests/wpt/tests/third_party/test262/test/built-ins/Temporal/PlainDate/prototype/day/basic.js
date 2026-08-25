// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindate.prototype.day
description: The "day" property of Temporal.PlainDate.prototype
features: [Temporal]
---*/

assert.sameValue((new Temporal.PlainDate(2021, 7, 15)).day, 15);
assert.sameValue(Temporal.PlainDate.from('2019-03-18').day, 18);
