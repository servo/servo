// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
for (var v of [-300, 255.6, 300, 3.5, -3.9]) {
    var a = new Uint8ClampedArray([v]);
    var b = new Uint8ClampedArray(1);
    b[0] = v;

    assert.sameValue(a[0], b[0]);
}

