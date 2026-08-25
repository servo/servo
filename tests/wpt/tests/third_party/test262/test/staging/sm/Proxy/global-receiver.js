// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// The global object can be the receiver passed to the get and set traps of a Proxy.
var global = this;
var proto = Object.getPrototypeOf(global);
var gets = 0, sets = 0;

try {
    Object.setPrototypeOf(global, new Proxy(proto, {
        has(t, id) {
            return id === "bareword" || Reflect.has(t, id);
        },
        get(t, id, r) {
            gets++;
            assert.sameValue(r, global);
            return Reflect.get(t, id, r);
        },
        set(t, id, v, r) {
            sets++;
            assert.sameValue(r, global);
            return Reflect.set(t, id, v, r);
        }
    }));
} catch (e) {
    global.bareword = undefined;
    gets = 1;
    sets = 1;
}

assert.sameValue(bareword, undefined);
assert.sameValue(gets, 1);

bareword = 12;
assert.sameValue(sets, 1);
assert.sameValue(global.bareword, 12);

