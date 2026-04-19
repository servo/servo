// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.from
description: Leap second is a valid ISO string for Instant
features: [Temporal]
---*/

const arg = "2016-12-31T23:59:60Z";
const result = Temporal.Instant.from(arg);
assert.sameValue(
  result.epochNanoseconds,
  1_483_228_799_000_000_000n,
  "leap second is a valid ISO string for Instant"
);
