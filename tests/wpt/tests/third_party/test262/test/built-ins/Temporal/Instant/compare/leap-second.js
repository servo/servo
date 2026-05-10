// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: Leap second is a valid ISO string for Instant
features: [Temporal]
---*/

const other = new Temporal.Instant(1_483_228_799_000_000_000n);
const arg = "2016-12-31T23:59:60Z";
const result1 = Temporal.Instant.compare(arg, other);
assert.sameValue(result1, 0, "leap second is a valid ISO string for Instant (first argument)");
const result2 = Temporal.Instant.compare(other, arg);
assert.sameValue(result2, 0, "leap second is a valid ISO string for Instant (second argument)");
