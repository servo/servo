// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js, detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/

function assertSameEntries(actual, expected) {
    assert.sameValue(actual.length, expected.length);
    for (let i = 0; i < expected.length; ++i)
        assert.compareArray(actual[i], expected[i]);
}

// Ensure Object.keys/values/entries work correctly on typed arrays.
for (let len of [0, 1, 10]) {
    let array = new Array(len).fill(1);
    let ta = new Int32Array(array);

    assert.compareArray(Object.keys(ta), Object.keys(array));
    assert.compareArray(Object.values(ta), Object.values(array));
    assertSameEntries(Object.entries(ta), Object.entries(array));

    $DETACHBUFFER(ta.buffer);

    assert.compareArray(Object.keys(ta), []);
    assert.compareArray(Object.values(ta), []);
    assertSameEntries(Object.entries(ta), []);
}
