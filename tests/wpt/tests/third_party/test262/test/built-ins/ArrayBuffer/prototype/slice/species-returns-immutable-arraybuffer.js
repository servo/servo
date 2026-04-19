// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Throws a TypeError if species constructor returns an immutable ArrayBuffer.
info: |
  ArrayBuffer.prototype.slice ( start, end )
  1. Let O be the this value.
  ...
  14. Let bounds be ? ResolveBounds(len, start, end).
  ...
  18. Let ctor be ? SpeciesConstructor(O, %ArrayBuffer%).
  19. Let new be ? Construct(ctor, « 𝔽(newLen) »).
  ...
  23. If IsImmutableBuffer(new) is true, throw a TypeError exception.
features: [Symbol.species, immutable-arraybuffer]
includes: [compareArray.js]
---*/

var calls = [];

var speciesConstructor = {};
speciesConstructor[Symbol.species] = function(length) {
  calls.push("Symbol.species(" + length + ")");
  return arrayBuffer.sliceToImmutable();
};

var arrayBuffer = new ArrayBuffer(8);
arrayBuffer.constructor = speciesConstructor;

assert.throws(TypeError, function() {
  arrayBuffer.slice();
});
assert.compareArray(calls, ["Symbol.species(8)"]);

calls = [];
assert.throws(TypeError, function() {
  var start = {
    valueOf() {
      calls.push("start.valueOf");
      return 1;
    }
  };
  var end = {
    valueOf() {
      calls.push("end.valueOf");
      return 2;
    }
  };
  arrayBuffer.slice(start, end);
});
assert.compareArray(calls, ["start.valueOf", "end.valueOf", "Symbol.species(1)"],
  "Must read arguments before SpeciesConstructor.");
