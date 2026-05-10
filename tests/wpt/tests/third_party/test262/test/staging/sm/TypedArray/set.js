// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, propertyHelper.js]
description: |
  pending
esid: pending
---*/
const TypedArrayPrototype = Object.getPrototypeOf(Int8Array.prototype);

// %TypedArrayPrototype% has an own "set" function property.
assert.sameValue(TypedArrayPrototype.hasOwnProperty("set"), true);
assert.sameValue(typeof TypedArrayPrototype.set, "function");

// The concrete TypedArray prototypes do not have an own "set" property.
assert.sameValue(anyTypedArrayConstructors.every(c => !c.hasOwnProperty("set")), true);

verifyProperty(TypedArrayPrototype, "set", {
    value: TypedArrayPrototype.set,
    writable: true,
    enumerable: false,
    configurable: true,
}, {
  restore: true
});

assert.sameValue(TypedArrayPrototype.set.name, "set");
assert.sameValue(TypedArrayPrototype.set.length, 1);
