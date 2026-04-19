// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/licenses/publicdomain/

// Check that |x % x| returns zero when |x| contains multiple digits
assert.sameValue(0x10000000000000000n % 0x10000000000000000n, 0n);
assert.sameValue(-0x10000000000000000n % -0x10000000000000000n, 0n);
