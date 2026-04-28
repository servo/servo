// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-typedarray-prototype-objects
description: BigUint64Array.prototype is not a TypedArray instance
info: |
  22.2.6 Properties of TypedArray Prototype Objects

  [...] A TypedArray prototype object is an ordinary object. It does not
  have a [[ViewedArrayBuffer]] or any other of the internal slots that
  are specific to TypedArray instance objects.
features: [BigInt]
---*/
assert.sameValue(typeof BigUint64Array, 'function');
assert.throws(TypeError, function () {
  BigUint64Array.prototype.buffer;
});
