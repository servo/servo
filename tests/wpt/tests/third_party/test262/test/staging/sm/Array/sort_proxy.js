// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

// Ensure, via proxy, that only get, set, delete, has, and getOwnPropertyDescriptor
// are touched during sorting.

const handler = {
    set: function(target, prop, value) {
        target[prop] = value;
        return value;
    },
    getPrototypeOf: () => { throw "You shouldn't touch getPrototypeOf!" },
    setPrototypeOf: () => { throw "You shouldn't touch setPrototypeOf!" },
    isExtensible: () => { throw "You shouldn't touch isExtensible!" },
    preventExtensions: () => { throw "You shouldn't touch preventExtensions!" },
    defineProperty: () => { throw "You shouldn't touch defineProperty!" },
    ownKeys: () => { throw "You shouldn't touch ownKeys!" },
    apply: () => { throw "You shouldn't touch apply!" },
    construct: () => { throw "You shouldn't touch construct!" },
}

function testArray(arr) {
    let proxy = new Proxy(arr, handler)

    // The supplied comparators trigger a JavaScript implemented sort.
    proxy.sort((x, y) => 1 * x - y);
    arr.sort((x, y) => 1 * x - y);

    for (let i in arr)
        assert.sameValue(arr[i], proxy[i]);
}

testArray([-1]);
testArray([5, -1, 2, 99]);
testArray([5, -1, , , , 2, 99]);
testArray([]);

