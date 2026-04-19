// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.fill
description: >
  Fills all the elements with non numeric values values.
info: |
  22.2.3.8 %TypedArray%.prototype.fill (value [ , start [ , end ] ] )

  %TypedArray%.prototype.fill is a distinct function that implements the same
  algorithm as Array.prototype.fill as defined in 22.1.3.6 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length". The implementation of the algorithm may be optimized with
  the knowledge that the this value is an object that has a fixed length and
  whose integer indexed properties are not sparse. However, such optimization
  must not introduce any observable changes in the specified behaviour of the
  algorithm.

  ...

  22.1.3.6 Array.prototype.fill (value [ , start [ , end ] ] )

  ...
  7. Repeat, while k < final
    a. Let Pk be ! ToString(k).
    b. Perform ? Set(O, Pk, value, true).
  ...

  24.1.1.6 SetValueInBuffer ( arrayBuffer, byteIndex, type, value [ ,
  isLittleEndian ] )

  ...
  8. If type is "Float32", then
    ...
  9. Else, if type is "Float64", then
    ...
  10. Else,
    ...
    b. Let convOp be the abstract operation named in the Conversion Operation
    column in Table 50 for Element Type type.
    c. Let intValue be convOp(value).
    d. If intValue ≥ 0, then
      ...
    e. Else,
      ...
includes: [byteConversionValues.js, testTypedArray.js]
features: [TypedArray]
---*/

testTypedArrayConversions(byteConversionValues, function(TA, value, expected, initial) {
  var sample = new TA([initial]);

  sample.fill(value);

  assert.sameValue(sample[0], expected, value + " converts to " + expected);
});
