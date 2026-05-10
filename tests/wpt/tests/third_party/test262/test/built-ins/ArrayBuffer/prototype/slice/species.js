// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  New ArrayBuffer instance is created from SpeciesConstructor.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  13. Let ctor be SpeciesConstructor(O, %ArrayBuffer%).
  14. ReturnIfAbrupt(ctor).
  15. Let new be Construct(ctor, «newLen»).
  16. ReturnIfAbrupt(new).
  ...
  26. Return new.
features: [Symbol.species]
---*/

var resultBuffer;

var speciesConstructor = {};
speciesConstructor[Symbol.species] = function(length) {
  return resultBuffer = new ArrayBuffer(length);
};

var arrayBuffer = new ArrayBuffer(8);
arrayBuffer.constructor = speciesConstructor;

var result = arrayBuffer.slice();
assert.sameValue(result, resultBuffer);
