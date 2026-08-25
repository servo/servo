// Copyright (c) 2017 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Including testTypedArray.js will expose:

        testTypedArrayConversions()

includes: [testTypedArray.js]
features: [TypedArray]
---*/
var callCount = 0;
var bcv = {
  values: [
    127,
  ],
  expected: {
    Int8: [
      127,
    ],
    Uint8: [
      127,
    ],
    Uint8Clamped: [
      127,
    ],
    Int16: [
      127,
    ],
    Uint16: [
      127,
    ],
    Int32: [
      127,
    ],
    Uint32: [
      127,
    ],
    Float16: [
      127,
    ],
    Float32: [
      127,
    ],
    Float64: [
      127,
    ]
  }
};

testTypedArrayConversions(bcv, function(TA, value, expected, initial) {
  var sample = new TA([initial]);
  sample.fill(value);
  assert.sameValue(initial, 0);
  assert.sameValue(sample[0], expected);
  callCount++;
});

