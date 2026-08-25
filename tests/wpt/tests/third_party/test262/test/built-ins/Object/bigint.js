// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Conversion of BigInt values to Objects
esid: pending
features: [BigInt]
---*/

assert(Object(0n) instanceof BigInt);
assert.sameValue(Object(0n).valueOf(), 0n);
