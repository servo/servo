// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function SortedAscending(length) {
    var array = new Int32Array(length);
    for (var i = 0; i < length; ++i)
        array[i] = i + 1;

    array.sort((x, y) => x - y);

    for (var i = 0; i < length; ++i)
        assert.sameValue(i + 1, array[i], `Mismatch at index=${i}, length=${length}`);
}

for (var i = 0; i < 256; ++i)
    SortedAscending(i);

function SortedDescending(length) {
    var array = new Int32Array(length);
    for (var i = 0; i < length; ++i)
        array[i] = length - i;

    array.sort((x, y) => x - y);

    for (var i = 0; i < length; ++i)
        assert.sameValue(i + 1, array[i], `Mismatch at index=${i}, length=${length}`);
}

for (var i = 0; i < 256; ++i)
    SortedDescending(i);

