// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-object
description: Consistent canonicalization of NaN values
info: |
  22.2.4.4 TypedArray ( object )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object does not have either a [[TypedArrayName]] or an [[ArrayBufferData]]
  internal slot.

  ...
  9. Repeat, while k < len
    ...
    c. Perform ? Set(O, Pk, kValue, true).
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

  24.1.1.6 SetValueInBuffer ( arrayBuffer, byteIndex, type, value [ ,
  isLittleEndian ] )

  ...
  8. If type is "Float32", then
     a. Set rawBytes to a List containing the 4 bytes that are the result
        of converting value to IEEE 754-2008 binary32 format using “Round to
        nearest, ties to even” rounding mode. If isLittleEndian is false, the
        bytes are arranged in big endian order. Otherwise, the bytes are
        arranged in little endian order. If value is NaN, rawValue may be set
        to any implementation chosen IEEE 754-2008 binary64 format Not-a-Number
        encoding. An implementation must always choose the same encoding for
        each implementation distinguishable NaN value.
  9. Else, if type is "Float64", then
     a. Set rawBytes to a List containing the 8 bytes that are the IEEE
        754-2008 binary64 format encoding of value. If isLittleEndian is false,
        the bytes are arranged in big endian order. Otherwise, the bytes are
        arranged in little endian order. If value is NaN, rawValue may be set
        to any implementation chosen IEEE 754-2008 binary32 format Not-a-Number
        encoding. An implementation must always choose the same encoding for
        each implementation distinguishable NaN value.
  ...
includes: [nans.js, testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function body(FloatArray) {
  var first = new FloatArray(NaNs);
  var second = new FloatArray(NaNs);
  var firstBytes = new Uint8Array(first.buffer);
  var secondBytes = new Uint8Array(second.buffer);

  assert(compareArray(firstBytes, secondBytes));
}, floatArrayConstructors);
