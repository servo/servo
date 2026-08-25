// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: RequireObjectCoercible for BigInt values
esid: pending
features: [BigInt]
---*/

try {
  let {} = 0n;
} catch (e) {
  throw new Test262Error('Expected RequireObjectCoercible to succeed for BigInt values');
}

assert.sameValue(Object.setPrototypeOf(0n, null), 0n);
