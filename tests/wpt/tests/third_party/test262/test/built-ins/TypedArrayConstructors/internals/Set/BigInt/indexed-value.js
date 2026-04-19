// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Returns true after setting value
info: |
  9.4.5.5 [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Perform ? IntegerIndexedElementSet(O, numericIndex, V).
      ii. Return true.
  ...

includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, Reflect, TypedArray]
---*/

let proto = TypedArray.prototype;
let throwDesc = {
  set: function() {
    throw new Test262Error('OrdinarySet was called!');
  }
};

Object.defineProperty(proto, '0', throwDesc);
Object.defineProperty(proto, '1', throwDesc);

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  let sample = new TA(makeCtorArg(2));
  assert.sameValue(Reflect.set(sample, '0', 1n), true, 'Reflect.set(sample, "0", 1n) must return true');
  assert.sameValue(sample[0], 1n, 'The value of sample[0] is 1n');
  assert.sameValue(Reflect.set(sample, '1', 42n), true, 'Reflect.set(sample, "1", 42n) must return true');
  assert.sameValue(sample[1], 42n, 'The value of sample[1] is 42n');
}, null, ["passthrough"]);
