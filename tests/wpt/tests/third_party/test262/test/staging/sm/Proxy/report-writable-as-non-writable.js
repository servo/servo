// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var target = {};
Object.defineProperty(target, "test",
    {configurable: false, writable: true, value: 1});

var proxy = new Proxy(target, {
    getOwnPropertyDescriptor(target, property) {
        assert.sameValue(property, "test");
        return {configurable: false, writable: false, value: 1};
    }
});

assert.throws(TypeError, () => Object.getOwnPropertyDescriptor(proxy, "test"));

assert.throws(TypeError, () => Reflect.getOwnPropertyDescriptor(proxy, "test"));
