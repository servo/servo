// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-hasproperty-p
description: Return abrupt from OrdinaryHasProperty parent's [[HasProperty]]
info: |
  9.4.5.2 [[HasProperty]](P)

  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Let buffer be O.[[ViewedArrayBuffer]].
      ii. If IsDetachedBuffer(buffer) is true, return false.
  ...

  9.1.7.1 OrdinaryHasProperty (O, P)

  ...
  2. Let hasOwn be ? O.[[GetOwnProperty]](P).
  3. If hasOwn is not undefined, return true.
  4. Let parent be ? O.[[GetPrototypeOf]]().
  5. If parent is not null, then
    a. Return ? parent.[[HasProperty]](P).
  6. Return false.
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, Reflect, Proxy, TypedArray]
---*/

var handler = {
  has: function() {
    throw new Test262Error();
  }
};

var proxy = new Proxy(TypedArray.prototype, handler);

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));

  Object.setPrototypeOf(sample, proxy);

  assert.sameValue(
    Reflect.has(sample, 0), true,
    'Reflect.has(sample, 0) must return true'
  );
  assert.sameValue(
    Reflect.has(sample, 1), false,
    'Reflect.has(sample, 1) must return false'
  );

  assert.throws(Test262Error, function() {
    Reflect.has(sample, "foo");
  });

  Object.defineProperty(sample, "foo", { value: 42 });

  assert.sameValue(
    Reflect.has(sample, "foo"),
    true,
    'Reflect.has(sample, "foo") must return true'
  );
}, null, ["passthrough"]);
