// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: Consistent canonicalization of NaN values
info: |
  This test does not compare the actual byte values, instead it simply checks that
  the value is some valid NaN encoding.

  ---

  [[DefineOwnProperty]] ( P, Desc)

  ...
  3. If Type(P) is String, then
    ...
    b. If numericIndex is not undefined, then
      ...
      xi. If Desc has a [[Value]] field, then
        1. Let value be Desc.[[Value]].
        2. Return ? IntegerIndexedElementSet(O, intIndex, value).
  ...

  #sec-integerindexedelementset
  IntegerIndexedElementSet ( O, index, value )

  ...
  15. Perform SetValueInBuffer(buffer, indexedPosition, elementType, numValue).
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
  var samples = new FA(makeCtorArg(1));
  var controls, idx, aNaN;

  for (idx = 0; idx < NaNs.length; ++idx) {
    aNaN = NaNs[idx];
    controls = new FA(makeCtorArg([aNaN, aNaN, aNaN]));

    Object.defineProperty(samples, "0", { value: aNaN });

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
}, floatArrayConstructors, null, ["immutable"]);
