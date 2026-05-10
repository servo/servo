// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.fill
description: >
  An implementation must always choose either the same encoding for each implementation distinguishable *NaN* value, or an implementation-defined canonical value.
info: |
  This test does not compare the actual byte values, instead it simply checks that
  the value is some valid NaN encoding.

  ---

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

  #sec-array.prototype.fill
  Array.prototype.fill (value [ , start [ , end ] ] )

  ...
  7. Repeat, while k < final
    a. Let Pk be ! ToString(k).
    b. Perform ? Set(O, Pk, value, true).
  ...

  #sec-setvalueinbuffer
  SetValueInBuffer ( arrayBuffer, byteIndex, type, value [ ,
  isLittleEndian ] )

  8. Let rawBytes be NumberToRawBytes(type, value, isLittleEndian).

  #sec-numbertorawbytes
  NumberToRawBytes( type, value, isLittleEndian )

  1. If type is "Float32", then
     a. Set rawBytes to a List containing the 4 bytes that are the result
        of converting value to IEEE 754-2008 binary32 format using “Round to
        nearest, ties to even” rounding mode. If isLittleEndian is false, the
        bytes are arranged in big endian order. Otherwise, the bytes are
        arranged in little endian order. If value is NaN, rawValue may be set
        to any implementation chosen IEEE 754-2008 binary64 format Not-a-Number
        encoding. An implementation must always choose either the same encoding
        for each implementation distinguishable *NaN* value, or an
        implementation-defined canonical value.
  2. Else, if type is "Float64", then
     a. Set _rawBytes_ to a List containing the 8 bytes that are the IEEE
        754-2008 binary64 format encoding of _value_. If _isLittleEndian_ is
        *false*, the bytes are arranged in big endian order. Otherwise,
        the bytes are arranged in little endian order. If _value_ is *NaN*,
        _rawValue_ may be set to any implementation chosen IEEE 754-2008
        binary64 format Not-a-Number encoding. An implementation must
        always choose either the same encoding for each implementation
        distinguishable *NaN* value, or an implementation-defined
        canonical value.
  ...

  #sec-isnan-number

  NOTE: A reliable way for ECMAScript code to test if a value X is a NaN is 
  an expression of the form  X !== X. The result will be true if and only 
  if X is a NaN.
includes: [nans.js, testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(FA, makeCtorArg) {
  var precision = floatTypedArrayConstructorPrecision(FA);
  var samples = new FA(makeCtorArg(3));
  var controls, idx, aNaN;

  for (idx = 0; idx < NaNs.length; ++idx) {
    aNaN = NaNs[idx];
    controls = new Float32Array([aNaN, aNaN, aNaN]);

    samples.fill(aNaN);

    for (var i = 0; i < samples.length; i++) {
      var sample = samples[i];
      var control = controls[i];

      assert(
        samples[i] !== samples[i],
        `samples (index=${idx}) produces a valid NaN (${precision} precision)`
      );

      assert(
        controls[i] !== controls[i],
        `controls (index=${idx}) produces a valid NaN (${precision} precision)`
      );
    }
  }
}, floatArrayConstructors);
