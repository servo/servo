// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class base { constructor() { } }

// lies and the lying liars who tell them
function lies() { }
lies.prototype = 4;

assert.throws(TypeError, ()=>Reflect.consruct(base, [], lies));

// lie a slightly different way
function get(target, property, receiver) {
    if (property === "prototype")
        return 42;
    return Reflect.get(target, property, receiver);
}

class inst extends base {
    constructor() { super(); }
}
assert.throws(TypeError, ()=>new new Proxy(inst, {get})());

class defaultInst extends base {}
assert.throws(TypeError, ()=>new new Proxy(defaultInst, {get})());

