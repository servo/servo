// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var ta = new Uint8Array(1);
ta[0] = 17;

assert.sameValue(ta.lastIndexOf(17, -0), +0);

