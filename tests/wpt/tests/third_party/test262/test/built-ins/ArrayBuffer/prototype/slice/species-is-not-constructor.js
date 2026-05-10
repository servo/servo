// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Throws a TypeError if species constructor is not a constructor function.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  13. Let ctor be SpeciesConstructor(O, %ArrayBuffer%).
  14. ReturnIfAbrupt(ctor).
  ...

  7.3.20 SpeciesConstructor ( O, defaultConstructor )
    ...
    6. Let S be Get(C, @@species).
    7. ReturnIfAbrupt(S).
    ...
    9. If IsConstructor(S) is true, return S.
    10. Throw a TypeError exception.
features: [Symbol.species]
---*/

var speciesConstructor = {};

var arrayBuffer = new ArrayBuffer(8);
arrayBuffer.constructor = speciesConstructor;

function callSlice() {
  arrayBuffer.slice();
}

speciesConstructor[Symbol.species] = {};
assert.throws(TypeError, callSlice, "`constructor[Symbol.species]` value is Object");

speciesConstructor[Symbol.species] = Function.prototype;
assert.throws(TypeError, callSlice, "`constructor[Symbol.species]` value is Function.prototype");
