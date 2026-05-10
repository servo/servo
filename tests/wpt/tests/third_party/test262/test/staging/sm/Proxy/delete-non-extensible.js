// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var target = { test: true };
Object.preventExtensions(target);

var proxy = new Proxy(target, {
    deleteProperty(target, property) {
        return true;
    }
});

assert.sameValue(delete proxy.missing, true);
assert.sameValue(Reflect.deleteProperty(proxy, "missing"), true);

assert.throws(TypeError, () => { delete proxy.test; });
assert.throws(TypeError, () => Reflect.deleteProperty(proxy, "test"));
