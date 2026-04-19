// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, compareArray.js]
description: |
  pending
esid: pending
---*/

// Test %TypedArray%.prototype.set(typedArray, offset) when called with wrapped
// typed array.

var otherGlobal = $262.createRealm().global;

function taintLengthProperty(obj) {
    Object.defineProperty(obj, "length", {
        get() {
            assert.sameValue(true, false);
        }
    });
}

for (var TA of anyTypedArrayConstructors) {
    var target = new TA(4);
    var source = new otherGlobal[TA.name]([10, 20]);

    // Ensure "length" getter accessor isn't called.
    taintLengthProperty(source);

    assert.compareArray(target, [0, 0, 0, 0]);
    target.set(source, 1);
    assert.compareArray(target, [0, 10, 20, 0]);
}

// Detachment checks are also applied correctly for wrapped typed arrays.

// Create typed array from different global (explicit constructor call).
for (var TA of typedArrayConstructors) {
    var target = new TA(4);
    var source = new otherGlobal[TA.name](1);
    taintLengthProperty(source);

    // Called with wrapped typed array, array buffer already detached.
    otherGlobal.$262.detachArrayBuffer(source.buffer);
    assert.throws(TypeError, () => target.set(source));

    var source = new otherGlobal[TA.name](1);
    taintLengthProperty(source);

    // Called with wrapped typed array, array buffer detached when
    // processing offset parameter.
    var offset = {
        valueOf() {
            otherGlobal.$262.detachArrayBuffer(source.buffer);
            return 0;
        }
    };
    assert.throws(TypeError, () => target.set(source, offset));
}

// Create typed array from different global (implictly created when
// ArrayBuffer is a CCW).
for (var TA of typedArrayConstructors) {
    var target = new TA(4);
    var source = new TA(new otherGlobal.ArrayBuffer(1 * TA.BYTES_PER_ELEMENT));
    taintLengthProperty(source);

    // Called with wrapped typed array, array buffer already detached.
    otherGlobal.$262.detachArrayBuffer(source.buffer);
    assert.throws(TypeError, () => target.set(source));

    var source = new TA(new otherGlobal.ArrayBuffer(1 * TA.BYTES_PER_ELEMENT));
    taintLengthProperty(source);

    // Called with wrapped typed array, array buffer detached when
    // processing offset parameter.
    var offset = {
        valueOf() {
            otherGlobal.$262.detachArrayBuffer(source.buffer);
            return 0;
        }
    };
    assert.throws(TypeError, () => target.set(source, offset));
}
