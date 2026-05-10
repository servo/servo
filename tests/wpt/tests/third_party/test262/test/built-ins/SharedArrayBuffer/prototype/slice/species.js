// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  New SharedArrayBuffer instance is created from SpeciesConstructor.
info: |
  SharedArrayBuffer.prototype.slice ( start, end )

features: [SharedArrayBuffer, Symbol.species]
---*/

var resultBuffer;

var speciesConstructor = {};
speciesConstructor[Symbol.species] = function(length) {
  return resultBuffer = new SharedArrayBuffer(length);
};

var arrayBuffer = new SharedArrayBuffer(8);
arrayBuffer.constructor = speciesConstructor;

var result = arrayBuffer.slice();
assert.sameValue(result, resultBuffer);
