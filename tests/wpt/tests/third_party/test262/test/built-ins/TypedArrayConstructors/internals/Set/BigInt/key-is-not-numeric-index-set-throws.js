// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Returns abrupt from OrdinarySet when key is not a numeric index
info: |
  9.4.5.5 [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
  ...
  3. Return ? OrdinarySet(O, P, V, Receiver).

  9.1.9.1 OrdinarySet (O, P, V, Receiver)

  ...
  8. Perform ? Call(setter, Receiver, « V »).
  ...
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));

  Object.defineProperty(sample, "test262", {
    set: function() {
      throw new Test262Error();
    }
  });

  assert.throws(Test262Error, function() {
    sample.test262 = 1;
  });

  assert.sameValue(sample.test262, undefined, 'The value of sample.test262 is expected to equal `undefined`');
}, null, ["passthrough"]);
