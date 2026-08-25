/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/
// Reflect.getOwnPropertyDescriptor inspects object properties.
assert.deepEqual(
    Reflect.getOwnPropertyDescriptor({x: "hello"}, "x"),
    {value: "hello", writable: true, enumerable: true, configurable: true});
assert.sameValue(
    Reflect.getOwnPropertyDescriptor({x: "hello"}, "y"),
    undefined);
assert.deepEqual(
    Reflect.getOwnPropertyDescriptor([], "length"),
    {value: 0, writable: true, enumerable: false, configurable: false});

// Reflect.getOwnPropertyDescriptor shares its implementation with
// Object.getOwnPropertyDescriptor. The only difference is how non-object
// targets are handled.
//
// For more Reflect.getOwnPropertyDescriptor tests, see target.js and propertyKeys.js.

