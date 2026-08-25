// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Epoch milliseconds should be rounded down before adding negative micro/nanoseconds back in
features: [Temporal]
---*/

const pdt = new Temporal.PlainDateTime(1938, 4, 24, 22, 13, 19, 999, 999);
assert.sameValue(pdt.toString(), "1938-04-24T22:13:19.999999",
                 "epoch milliseconds should be rounded down to compute seconds");
