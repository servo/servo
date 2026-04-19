// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-get-p-receiver
description: >
  Returns abrupt from OrdinaryGet when key is not a numeric index
info: |
  9.4.5.4 [[Get]] (P, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      ...
  3. Return ? OrdinaryGet(O, P, Receiver).

  9.1.8.1 OrdinaryGet (O, P, Receiver)

  ...
  8. Return ? Call(getter, Receiver).
  ...
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));

  Object.defineProperty(sample, "test262", {
    get: function() {
      throw new Test262Error();
    }
  });

  assert.throws(Test262Error, function() {
    sample.test262;
  });
}, null, ["passthrough"]);
