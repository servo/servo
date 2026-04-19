// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/
// The prototype of a class is a non-writable, non-configurable, non-enumerable data property.
class a { constructor() { } }
let b = class { constructor() { } };
for (let test of [a,b]) {
    var protoDesc = Object.getOwnPropertyDescriptor(test, "prototype");
    assert.sameValue(protoDesc.writable, false);
    assert.sameValue(protoDesc.configurable, false);
    assert.sameValue(protoDesc.enumerable, false);

    var prototype = protoDesc.value;
    assert.sameValue(typeof prototype, "object");
    assert.sameValue(Object.getPrototypeOf(prototype), Object.prototype);
    assert.sameValue(Object.isExtensible(prototype), true);

    var desiredPrototype = {};
    Object.defineProperty(desiredPrototype, "constructor", { writable: true,
                                                            configurable: true,
                                                            enumerable: false,
                                                            value: test });
    assert.deepEqual(prototype, desiredPrototype);
}

// As such, it should by a TypeError to try and overwrite "prototype" with a
// static member. The only way to try is with a computed property name; the rest
// are early errors.
assert.throws(TypeError, () => eval(`
                                  class a {
                                    constructor() { };
                                    static ["prototype"]() { }
                                  }
                                  `));
assert.throws(TypeError, () => eval(`
                                  class a {
                                    constructor() { };
                                    static get ["prototype"]() { }
                                  }
                                  `));
assert.throws(TypeError, () => eval(`
                                  class a {
                                    constructor() { };
                                    static set ["prototype"](x) { }
                                  }
                                  `));

assert.throws(TypeError, () => eval(`(
                                  class a {
                                    constructor() { };
                                    static ["prototype"]() { }
                                  }
                                  )`));
assert.throws(TypeError, () => eval(`(
                                  class a {
                                    constructor() { };
                                    static get ["prototype"]() { }
                                  }
                                  )`));
assert.throws(TypeError, () => eval(`(
                                  class a {
                                    constructor() { };
                                    static set ["prototype"](x) { }
                                  }
                                  )`));
