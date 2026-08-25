// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Throws a TypeError if new SharedArrayBuffer is too small.
info: |
  SharedArrayBuffer.prototype.slice ( start, end )

features: [SharedArrayBuffer, Symbol.species]
---*/

var speciesConstructor = {};
speciesConstructor[Symbol.species] = function(length) {
  return new SharedArrayBuffer(4);
};

var arrayBuffer = new SharedArrayBuffer(8);
arrayBuffer.constructor = speciesConstructor;

assert.throws(TypeError, function() {
  arrayBuffer.slice();
});
