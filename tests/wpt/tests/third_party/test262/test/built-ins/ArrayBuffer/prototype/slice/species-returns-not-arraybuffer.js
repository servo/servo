// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Throws a TypeError if new object is not an ArrayBuffer instance.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  13. Let ctor be SpeciesConstructor(O, %ArrayBuffer%).
  14. ReturnIfAbrupt(ctor).
  15. Let new be Construct(ctor, «newLen»).
  16. ReturnIfAbrupt(new).
  17. If new does not have an [[ArrayBufferData]] internal slot, throw a TypeError exception.
  ...
features: [Symbol.species]
---*/

var speciesConstructor = {};
speciesConstructor[Symbol.species] = function(length) {
  return {};
};

var arrayBuffer = new ArrayBuffer(8);
arrayBuffer.constructor = speciesConstructor;

assert.throws(TypeError, function() {
  arrayBuffer.slice();
});
