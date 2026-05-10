// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Throws a TypeError if species constructor returns `this` value.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  1. Let O be the this value.
  ...
  13. Let ctor be SpeciesConstructor(O, %ArrayBuffer%).
  14. ReturnIfAbrupt(ctor).
  15. Let new be Construct(ctor, «newLen»).
  16. ReturnIfAbrupt(new).
  ...
  19. If SameValue(new, O) is true, throw a TypeError exception.
  ...
features: [Symbol.species]
---*/

var speciesConstructor = {};
speciesConstructor[Symbol.species] = function(length) {
  return arrayBuffer;
};

var arrayBuffer = new ArrayBuffer(8);
arrayBuffer.constructor = speciesConstructor;

assert.throws(TypeError, function() {
  arrayBuffer.slice();
});
