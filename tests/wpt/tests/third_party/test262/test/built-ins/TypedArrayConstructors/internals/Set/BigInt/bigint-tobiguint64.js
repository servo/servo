// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: >
  Behavior for input array of BigInts
info: |
  Runtime Semantics: Evaluation
  AssignmentExpression : LeftHandSideExpression = AssignmentExpression
  1. If LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral, then
  ...
    f. Perform ? PutValue(lref, rval).
  ...

  PutValue ( V, W )
  ...
  6. Else if IsPropertyReference(V) is true, then
    a. If HasPrimitiveBase(V) is true, then
        i. Assert: In this case, base will never be undefined or null.
        ii. Set base to ! ToObject(base).
    b. Let succeeded be ? base.[[Set]](GetReferencedName(V), W, GetThisValue(V)).
    c. If succeeded is false and IsStrictReference(V) is true, throw a TypeError
       exception.
    d. Return.

  [[Set]] ( P, V, Receiver )
  When the [[Set]] internal method of an Integer-Indexed exotic object O is
  called with property key P, value V, and ECMAScript language value Receiver,
  the following steps are taken:
  1. Assert: IsPropertyKey(P) is true.
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
       i. Return ? IntegerIndexedElementSet(O, numericIndex, V).

  IntegerIndexedElementSet ( O, index, value )
  5. If arrayTypeName is "BigUint64Array" or "BigInt64Array",
     let numValue be ? ToBigInt(value).
  ...
  16. Perform SetValueInBuffer(buffer, indexedPosition, elementType, numValue, true, "Unordered").
  17. Return true.

  SetValueInBuffer ( arrayBuffer, byteIndex, type, value, isTypedArray, order [ , isLittleEndian ] )
  ...
  8. Let rawBytes be NumberToRawBytes(type, value, isLittleEndian).
  ...

  NumberToRawBytes( type, value, isLittleEndian )
  ...
  3. Else,
    a. Let n be the Number value of the Element Size specified in Table
       [The TypedArray Constructors] for Element Type type.
    b. Let convOp be the abstract operation named in the Conversion Operation
       column in Table 9 for Element Type type.

  The TypedArray Constructors
  Element Type: BigUint64
  Conversion Operation: ToBigUint64

  ToBigUint64 ( argument )
  The abstract operation ToBigInt64 converts argument to one of 264 integer
  values in the range -2^63 through 2^63-1, inclusive.
  This abstract operation functions as follows:
    1. Let n be ? ToBigInt(argument).
    2. Let int64bit be n modulo 2^64.
    3. Return int64bit.

features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/
// 2n ** 64n + 2n
// 2n ** 63n + 2n
// -(2n ** 63n) - 2n
// -(2n ** 64n) - 2n
// 2n ** 63n + 2n
// 2n ** 64n - 2n
// 2n ** 63n - 2n
// 2n ** 64n - 2n
var vals = [
  18446744073709551618n,
  9223372036854775810n,
  2n,
  0n,
  -2n,
  -9223372036854775810n,
  -18446744073709551618n
];

var typedArray = new BigUint64Array(1);
typedArray[0] = vals[0];
assert.sameValue(typedArray[0], 2n, 'The value of typedArray[0] is 2n');
typedArray[0] = vals[1];
assert.sameValue(typedArray[0], 9223372036854775810n, 'The value of typedArray[0] is 9223372036854775810n');
typedArray[0] = vals[2];
assert.sameValue(typedArray[0], 2n, 'The value of typedArray[0] is 2n');
typedArray[0] = vals[3];
assert.sameValue(typedArray[0], 0n, 'The value of typedArray[0] is 0n');
typedArray[0] = vals[4];
assert.sameValue(typedArray[0], 18446744073709551614n, 'The value of typedArray[0] is 18446744073709551614n');
typedArray[0] = vals[5];
assert.sameValue(typedArray[0], 9223372036854775806n, 'The value of typedArray[0] is 9223372036854775806n');
typedArray[0] = vals[6];
assert.sameValue(typedArray[0], 18446744073709551614n, 'The value of typedArray[0] is 18446744073709551614n');
