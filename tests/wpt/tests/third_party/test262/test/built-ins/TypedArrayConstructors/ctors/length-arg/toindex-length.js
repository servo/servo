// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-length
description: >
  ToIndex(length) operations
info: |
  22.2.4.2 TypedArray ( length )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is not Object.

  ...
  3. Let elementLength be ? ToIndex(length).
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var items = [
  [-0, 0, "-0"],
  ["", 0, "the Empty string"],
  ["0", 0, "string '0'"],
  ["1", 1, "string '1'"],
  [true, 1, "true"],
  [false, 0, "false"],
  [NaN, 0, "NaN"],
  [null, 0, "null"],
  [undefined, 0, "undefined"],
  [0.1, 0, "0.1"],
  [0.9, 0, "0.9"],
  [1.1, 1, "1.1"],
  [1.9, 1, "1.9"],
  [-0.1, 0, "-0.1"],
  [-0.99999, 0, "-0.99999"]
];

testWithTypedArrayConstructors(function(TA) {
  items.forEach(function(item) {
    var len = item[0];
    var expected = item[1];
    var name = item[2];

    var typedArray = new TA(len);
    assert.sameValue(typedArray.length, expected, name + " length");
    assert.sameValue(typedArray.constructor, TA, name + " constructor");
    assert.sameValue(
      Object.getPrototypeOf(typedArray),
      TA.prototype,
      name + " prototype"
    );
  });
}, null, ["passthrough"]);
