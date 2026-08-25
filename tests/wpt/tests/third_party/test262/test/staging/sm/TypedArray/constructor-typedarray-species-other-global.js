// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
// 22.2.4.3 TypedArray ( typedArray )

// Test [[Prototype]] of newly created typed array and its array buffer, and
// ensure they are both created in the correct global.

const thisGlobal = this;
const otherGlobal = $262.createRealm().global;

const typedArrays = [otherGlobal.eval("new Int32Array(0)")];

if (this.SharedArrayBuffer) {
    typedArrays.push(otherGlobal.eval("new Int32Array(new SharedArrayBuffer(0))"));
}

for (let typedArray of typedArrays) {
    // Ensure the "constructor" property isn't accessed.
    Object.defineProperty(typedArray.buffer, "constructor", {
        get() {
            throw new Error("constructor property accessed");
        }
    });

    for (let ctor of typedArrayConstructors) {
        let newTypedArray = new ctor(typedArray);

        assert.sameValue(Object.getPrototypeOf(newTypedArray), ctor.prototype);
        assert.sameValue(Object.getPrototypeOf(newTypedArray.buffer), ArrayBuffer.prototype);
        assert.sameValue(newTypedArray.buffer.constructor, ArrayBuffer);
    }
}

