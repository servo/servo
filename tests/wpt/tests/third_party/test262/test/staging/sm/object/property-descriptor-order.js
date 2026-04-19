// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/

var names = Object.getOwnPropertyNames(Object.getOwnPropertyDescriptor({foo: 0}, "foo"));
assert.compareArray(names, ["value", "writable", "enumerable", "configurable"]);

names = Object.getOwnPropertyNames(Object.getOwnPropertyDescriptor({get foo(){}}, "foo"));
assert.compareArray(names, ["get", "set", "enumerable", "configurable"]);

var proxy = new Proxy({}, {
    defineProperty(target, key, desc) {
        var names = Object.getOwnPropertyNames(desc);
        assert.compareArray(names, ["set", "configurable"]);
        return true;
    }
});

Object.defineProperty(proxy, "foo", {configurable: true, set: function() {}});

