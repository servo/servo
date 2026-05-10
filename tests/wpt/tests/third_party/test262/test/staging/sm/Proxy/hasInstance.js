// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
var get = [];
var fun = function() {}
var p = new Proxy(fun, {
    get(target, key) {
        get.push(key);
        return target[key];
    }
});

assert.sameValue(new fun instanceof p, true);
assert.compareArray(get, [Symbol.hasInstance, "prototype"]);

