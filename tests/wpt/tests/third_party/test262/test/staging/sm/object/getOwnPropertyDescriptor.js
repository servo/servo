/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [deepEqual.js]
description: |
  Coerce the argument passed to Object.getOwnPropertyDescriptor using ToObject
info: bugzilla.mozilla.org/show_bug.cgi?id=1079188
esid: pending
features: [Symbol]
---*/

assert.throws(TypeError, () => Object.getOwnPropertyDescriptor());
assert.throws(TypeError, () => Object.getOwnPropertyDescriptor(undefined));
assert.throws(TypeError, () => Object.getOwnPropertyDescriptor(null));

Object.getOwnPropertyDescriptor(1);
Object.getOwnPropertyDescriptor(true);
Object.getOwnPropertyDescriptor(Symbol("foo"));

assert.deepEqual(Object.getOwnPropertyDescriptor("foo", "length"), {
    value: 3,
    writable: false,
    enumerable: false,
    configurable: false
});

assert.deepEqual(Object.getOwnPropertyDescriptor("foo", 0), {
    value: "f",
    writable: false,
    enumerable: true,
    configurable: false
});

