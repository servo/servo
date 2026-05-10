// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/

var objects = [
    Math.sin.bind(null),
    new Proxy(Math.sin.bind(null), {}),
    Function.prototype.bind.call(new Proxy(Math.sin, {}))
]

for (var obj of objects) {
    // Target is not constructable, so a new array should be created internally.
    assert.compareArray(Array.from.call(obj, [1, 2, 3]), [1, 2, 3]);
    assert.compareArray(Array.of.call(obj, 1, 2, 3), [1, 2, 3]);

    // Make sure they are callable, but not constructable.
    obj();
    assert.throws(TypeError, () => new obj);
}

