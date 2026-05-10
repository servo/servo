// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - onlyStrict
description: |
  pending
esid: pending
---*/

var y = new Proxy({}, {
    getOwnPropertyDescriptor(target, key) {
        if (key === "a") {
            return { configurable: true, get: function(v) {} };
        } else {
            assert.sameValue(key, "b");
            return { configurable: true, writable: false, value: 15 };
        }
    },

    defineProperty() {
        throw "not invoked";
    }
})

// This will invoke [[Set]] on the target, with the proxy as receiver.
assert.throws(TypeError, () => y.a = 1);
assert.throws(TypeError, () => y.b = 2);
