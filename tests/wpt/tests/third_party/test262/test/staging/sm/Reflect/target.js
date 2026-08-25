/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-Reflect-shell.js]
description: |
  pending
esid: pending
---*/
// Check correct handling of the `target` argument shared by every Reflect method.

// For each standard Reflect method, an array of arguments
// that would be OK after a suitable target argument.
var methodInfo = {
    apply: [undefined, []],
    construct: [[]],
    defineProperty: ["x", {}],
    deleteProperty: ["x"],
    get: ["x", {}],
    getOwnPropertyDescriptor: ["x"],
    getPrototypeOf: [],
    has: ["x"],
    isExtensible: [],
    ownKeys: [],
    preventExtensions: [],
    set: ["x", 0],
    setPrototypeOf: [{}]
};

// Check that all Reflect properties are listed above.
for (const name of Reflect.ownKeys(Reflect)) {
    // If this assertion fails, congratulations on implementing a new Reflect feature!
    // Add it to methodInfo above.
    if (typeof name !== "symbol" && name !== "parse")
      assert.sameValue(name in methodInfo, true);
}

for (const name of Object.keys(methodInfo)) {
    var args = methodInfo[name];

    // The target argument is required.
    assert.throws(TypeError, Reflect[name]);

    // Throw if the target argument is not an object.
    for (var value of SOME_PRIMITIVE_VALUES) {
        assert.throws(TypeError, () => Reflect[name](value, ...args));
    }
}

