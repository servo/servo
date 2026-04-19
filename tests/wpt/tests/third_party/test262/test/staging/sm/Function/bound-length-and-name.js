// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var proxy = new Proxy(function() {}, {
    getOwnPropertyDescriptor(target, name) {
        assert.sameValue(name, "length");
        return {value: 3, configurable: true};
    },

    get(target, name) {
        if (name == "length")
            return 3;
        if (name == "name")
            return "hello world";
        assert.sameValue(false, true);
    }
})

var bound = Function.prototype.bind.call(proxy);
assert.sameValue(bound.name, "bound hello world");
assert.sameValue(bound.length, 3);

var fun = function() {};
Object.defineProperty(fun, "name", {value: 1337});
Object.defineProperty(fun, "length", {value: "15"});
bound = fun.bind();
assert.sameValue(bound.name, "bound ");
assert.sameValue(bound.length, 0);

Object.defineProperty(fun, "length", {value: Number.MAX_SAFE_INTEGER});
bound = fun.bind();
assert.sameValue(bound.length, Number.MAX_SAFE_INTEGER);

Object.defineProperty(fun, "length", {value: -100});
bound = fun.bind();
assert.sameValue(bound.length, 0);

fun = function f(a, ...b) { };
assert.sameValue(fun.length, 1);
bound = fun.bind();
assert.sameValue(bound.length, 1);

