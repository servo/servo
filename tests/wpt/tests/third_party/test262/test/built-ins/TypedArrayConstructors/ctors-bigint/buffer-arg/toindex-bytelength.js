// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: >
  ToIndex(length) operations
info: |
  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has an [[ArrayBufferData]] internal slot.

  ...
  11. If length is either not present or undefined, then
    ...
  12. Else,
    a. Let newLength be ? ToIndex(length).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var buffer = new ArrayBuffer(16);

var obj1 = {
  valueOf: function() {
    return 1;
  }
};

var obj2 = {
  toString: function() {
    return 1;
  }
};

var items = [
  [-0, 0, "-0"],
  [obj1, 1, "object's valueOf"],
  [obj2, 1, "object's toString"],
  ["", 0, "the Empty string"],
  ["0", 0, "string '0'"],
  ["1", 1, "string '1'"],
  [false, 0, "false"],
  [true, 1, "true"],
  [NaN, 0, "NaN"],
  [null, 0, "null"],
  [0.1, 0, "0.1"],
  [0.9, 0, "0.9"],
  [1.1, 1, "1.1"],
  [1.9, 1, "1.9"],
  [-0.1, 0, "-0.1"],
  [-0.99999, 0, "-0.99999"]
];

testWithBigIntTypedArrayConstructors(function(TA) {
  items.forEach(function(item) {
    var len = item[0];
    var expected = item[1];
    var name = item[2];

    var typedArray = new TA(buffer, 0, len);
    assert.sameValue(typedArray.length, expected, name + " length");
    assert.sameValue(typedArray.constructor, TA, name + " constructor");
    assert.sameValue(
      Object.getPrototypeOf(typedArray),
      TA.prototype,
      name + " prototype"
    );
  });
}, null, ["passthrough"]);
