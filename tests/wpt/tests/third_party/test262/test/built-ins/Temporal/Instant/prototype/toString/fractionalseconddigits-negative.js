// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Epoch milliseconds should be rounded down before adding negative micro/nanoseconds back in
features: [BigInt, Temporal]
---*/

const instant = new Temporal.Instant(-1000000000000001000n);
assert.sameValue(instant.toString(), "1938-04-24T22:13:19.999999Z",
                 "epoch milliseconds should be rounded down to compute seconds");
