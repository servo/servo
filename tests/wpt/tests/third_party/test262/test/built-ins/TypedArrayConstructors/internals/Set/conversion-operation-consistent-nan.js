// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: Consistent canonicalization of NaN values
info: |
  This test does not compare the actual byte values, instead it simply checks that
  the value is some valid NaN encoding.

  ---

  [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
    ...
    b. If numericIndex is not undefined, then
      i. Perform ? IntegerIndexedElementSet(O, numericIndex, V).
      ii. Return true.
  ...

  IntegerIndexedElementSet ( O, index, value )

  Assert: O is an Integer-Indexed exotic object.
  If O.[[ContentType]] is BigInt, let numValue be ? ToBigInt(value).
  Otherwise, let numValue be ? ToNumber(value).
  Let buffer be O.[[ViewedArrayBuffer]].
  If IsDetachedBuffer(buffer) is false and ! IsValidIntegerIndex(O, index) is true, then
    Let offset be O.[[ByteOffset]].
    Let arrayTypeName be the String value of O.[[TypedArrayName]].
    Let elementSize be the Element Size value specified in Table 62 for arrayTypeName.
    Let indexedPosition be (ℝ(index) × elementSize) + offset.
    Let elementType be the Element Type value in Table 62 for arrayTypeName.
    Perform SetValueInBuffer(buffer, indexedPosition, elementType, numValue, true, Unordered).
  Return NormalCompletion(undefined).

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
features: [align-detached-buffer-semantics-with-web-reality, TypedArray]
---*/

testWithTypedArrayConstructors(function(FA, makeCtorArg) {
  var precision = floatTypedArrayConstructorPrecision(FA);
  var samples = new FA(makeCtorArg(1));
  var controls, idx, aNaN;

  for (idx = 0; idx < NaNs.length; ++idx) {
    aNaN = NaNs[idx];
    controls = new FA(makeCtorArg([aNaN, aNaN, aNaN]));

    samples[0] = aNaN;

    for (var i = 0; i < samples.length; i++) {
      var sample = samples[i];
      var control = controls[i];

      assert(
        samples[i] !== samples[i],
        `The result of \`(samples[i] !== samples[i])\` is true (${precision} precision)`
      );

      assert(
        controls[i] !== controls[i],
        `The result of \`(controls[i] !== controls[i])\` is true (${precision} precision)`
      );
    }
  }
}, floatArrayConstructors, null, ["immutable"]);
