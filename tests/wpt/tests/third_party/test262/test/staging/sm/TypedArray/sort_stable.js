// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Test with different lengths to cover the case when InsertionSort is resp.
// is not called.
for (let i = 2; i <= 10; ++i) {
    let length = 2 ** i;
    let ta = new Int8Array(length);

    ta[0] = 2;
    ta[1] = 1;
    ta[2] = 0;

    for (let i = 3; i < length; ++i) {
        ta[i] = 4;
    }

    ta.sort((a, b) => (a/4|0) - (b/4|0));

    assert.sameValue(ta[0], 2);
    assert.sameValue(ta[1], 1);
    assert.sameValue(ta[2], 0);
}

