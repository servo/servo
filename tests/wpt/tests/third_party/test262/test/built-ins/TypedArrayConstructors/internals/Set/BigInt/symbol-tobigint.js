// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: >
  Return abrupt on Symbol
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

  ToBigInt ( argument )
  Object, Apply the following steps:
    1. Let prim be ? ToPrimitive(argument, hint Number).
    2. Return the value that prim corresponds to in Table [BigInt Conversions]

  BigInt Conversions
    Argument Type: Symbol
    Result: Throw a TypeError exception.

includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray, Symbol]
---*/

var s = Symbol()

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var typedArray = new TA(makeCtorArg(1))

  assert.throws(TypeError, function() {
    typedArray[0] = s;
  });

});
