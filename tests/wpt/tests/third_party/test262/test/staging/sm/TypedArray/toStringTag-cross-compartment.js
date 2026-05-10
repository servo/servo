// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
const TypedArrayPrototype = Object.getPrototypeOf(Int8Array.prototype);
const {get: toStringTag} = Object.getOwnPropertyDescriptor(TypedArrayPrototype, Symbol.toStringTag);

const otherGlobal = $262.createRealm().global;

for (let constructor of anyTypedArrayConstructors) {
    let ta = new otherGlobal[constructor.name](0);
    assert.sameValue(toStringTag.call(ta), constructor.name);
}

