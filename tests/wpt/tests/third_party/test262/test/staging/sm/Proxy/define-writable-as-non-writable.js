// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var target = {};
Object.defineProperty(target, "test", {configurable: false, writable: true, value: 5});

var proxy = new Proxy(target, {
    defineProperty(target, property) {
        assert.sameValue(property, "test");
        return true;
    }
});

assert.throws(TypeError, () => Object.defineProperty(proxy, "test", {writable: false}));

assert.throws(TypeError, () => Reflect.defineProperty(proxy, "test", {writable: false}));
