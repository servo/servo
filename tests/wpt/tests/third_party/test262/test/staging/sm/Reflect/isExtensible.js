/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/assertThrowsValue.js]
description: |
  pending
esid: pending
---*/
// Reflect.isExtensible behaves just like Object.extensible except when the
// target argument is missing or is not an object (and that behavior is tested
// in target.js).

// Test basic functionality.
var someObjects = [
    {},
    {a: "a"},
    [0, 1],
    new Uint8Array(64),
    Object(Symbol("table")),
    new Proxy({}, {})
];
if (typeof SharedArrayBuffer != "undefined")
  someObjects.push(new Uint8Array(new SharedArrayBuffer(64)));

for (var obj of someObjects) {
    assert.sameValue(Reflect.isExtensible(obj), true);
    assert.sameValue(Reflect.preventExtensions(obj), true);
    assert.sameValue(Reflect.isExtensible(obj), false);
}

// Array with nonwritable length.
var arr = [0, 1, 2, 3];
Object.defineProperty(arr, "length", {writable: false});
assert.sameValue(Reflect.isExtensible(arr), true);

// Proxy case.
for (var ext of [true, false]) {
    var obj = {};
    if (!ext)
        Object.preventExtensions(obj);
    var proxy = new Proxy(obj, {
        isExtensible() { return ext; }
    });
    assert.sameValue(Reflect.isExtensible(proxy), ext);
}

// If a Proxy's isExtensible method throws, the exception is propagated.
proxy = new Proxy({}, {
    isExtensible() { throw "oops"; }
});
assertThrowsValue(() => Reflect.isExtensible(proxy), "oops");

// If an invariant is broken, [[IsExtensible]] does not return false. It throws
// a TypeError.
proxy = new Proxy({}, {
    isExtensible() { return false; }
});
assert.throws(TypeError, () => Reflect.isExtensible(proxy));

// For more Reflect.isExtensible tests, see target.js.

