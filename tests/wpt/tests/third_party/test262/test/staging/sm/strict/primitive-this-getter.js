// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
let primitives = [
    10,
    false,
    "test",
    Symbol()
]

let getter = "getter";
let getter2 = "getter2";
let key = "key";

for (let value of primitives) {
    let prototype = Object.getPrototypeOf(value);

    // Strict getters receive a primitive this value.
    Object.defineProperty(prototype, "getter", {get: function() {
        "use strict";
        assert.sameValue(this, value);
        return "getter";
    }})

    assert.sameValue(value.getter, "getter");
    assert.sameValue(value[getter], "getter");

    // The proxy's [[Get]] trap is also invoked with primitive receiver values.
    let proxy = new Proxy({}, {
        get(target, property, receiver) {
            assert.sameValue(property, "key");
            assert.sameValue(receiver, value);
            return "get";
        }
    });

    Object.setPrototypeOf(prototype, proxy);
    assert.sameValue(value.key, "get");
    assert.sameValue(value[key], "get");
    assert.sameValue(value.getter, "getter");
    assert.sameValue(value[getter], "getter");

    // A getter still gets a primitive this value even after going through a proxy.
    proxy = new Proxy({
        get getter2() {
            "use strict";
            assert.sameValue(this, value);
            return "getter2";
        }
    }, {});

    Object.setPrototypeOf(prototype, proxy);
    assert.sameValue(value.getter2, "getter2");
    assert.sameValue(value[getter2], "getter2");
    assert.sameValue(value.getter, "getter");
    assert.sameValue(value[getter], "getter");
}

