// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Returns abrupt from ToNumber(value)
info: |
  9.4.5.5 [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Perform ? IntegerIndexedElementSet(O, numericIndex, V).
      ii. Return true.
  ...

  IntegerIndexedElementSet ( O, index, value )

  Assert: O is an Integer-Indexed exotic object.
  Assert: Type(index) is Number.
  If O.[[ContentType]] is BigInt, let numValue be ? ToBigInt(value).
  Otherwise, let numValue be ? ToNumber(value).
  ...
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/
testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  let sample = new TA(makeCtorArg([42n]));

  let obj = {
    valueOf() {
      throw new Test262Error();
    }
  };

  assert.throws(Test262Error, function() {
    sample['0'] = obj;
  }, '`sample["0"] = obj` throws Test262Error');

  assert.throws(Test262Error, function() {
    sample['1.1'] = obj;
  }, '`sample["1.1"] = obj` throws Test262Error');

  assert.throws(Test262Error, function() {
    sample['-0'] = obj;
  }, '`sample["-0"] = obj` throws Test262Error');

  assert.throws(Test262Error, function() {
    sample['-1'] = obj;
  }, '`sample["-1"] = obj` throws Test262Error');

  assert.throws(Test262Error, function() {
    sample['1'] = obj;
  }, '`sample["1"] = obj` throws Test262Error');

  assert.throws(Test262Error, function() {
    sample['2'] = obj;
  }, '`sample["2"] = obj` throws Test262Error');
}, null, null, ["immutable"]);
