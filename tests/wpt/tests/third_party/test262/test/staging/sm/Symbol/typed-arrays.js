/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// Symbol-to-number type conversions involving typed arrays.

for (var T of [Uint8Array, Uint8ClampedArray, Int16Array, Float32Array]) {
    // Typed array constructors convert symbols using ToNumber(), which throws.
    assert.throws(TypeError, () => new T([Symbol("a")]));

    // Assignment does the same.
    var arr = new T([1]);
    assert.throws(TypeError, () => { arr[0] = Symbol.iterator; });
    assert.sameValue(arr[0], 1);
}

