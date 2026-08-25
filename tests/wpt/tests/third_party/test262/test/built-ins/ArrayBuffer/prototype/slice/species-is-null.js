// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Uses default constructor is species constructor is null.
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
    8. If S is either undefined or null, return defaultConstructor.
    ...
features: [Symbol.species]
---*/

var speciesConstructor = {};
speciesConstructor[Symbol.species] = null;

var arrayBuffer = new ArrayBuffer(8);
arrayBuffer.constructor = speciesConstructor;

var result = arrayBuffer.slice();
assert.sameValue(Object.getPrototypeOf(result), ArrayBuffer.prototype);
