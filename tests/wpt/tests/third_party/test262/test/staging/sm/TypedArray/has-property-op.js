// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
for (var constructor of anyTypedArrayConstructors) {
    var obj = new constructor(5);

    for (var i = 0; i < obj.length; i++)
        assert.sameValue(i in obj, true);

    for (var v of [20, 300, -1, 5, -10, Math.pow(2, 32) - 1, -Math.pow(2, 32)])
        assert.sameValue(v in obj, false);

    // Don't inherit elements
    obj.__proto__[50] = "hello";
    assert.sameValue(obj.__proto__[50], "hello");
    assert.sameValue(50 in obj, false);

    // Do inherit normal properties
    obj.__proto__.a = "world";
    assert.sameValue(obj.__proto__.a, "world");
    assert.sameValue("a" in obj, true);
}

