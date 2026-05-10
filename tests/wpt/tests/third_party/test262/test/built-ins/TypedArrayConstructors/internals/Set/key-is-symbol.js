// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Use OrdinarySet if key is a Symbol
info: |
  9.4.5.5 [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
  ...
  3. Return ? OrdinarySet(O, P, V, Receiver).
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, Reflect, Symbol, TypedArray]
---*/

var s1 = Symbol("1");
var s2 = Symbol("2");

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42]));

  assert.sameValue(
    Reflect.set(sample, s1, "ecma262"),
    true,
    'Reflect.set(sample, "Symbol(\\"1\\")", "ecma262") must return true'
  );
  assert.sameValue(sample[s1], "ecma262", 'The value of sample[s1] is "ecma262"');

  assert.sameValue(
    Reflect.set(sample, s1, "es3000"),
    true,
    'Reflect.set(sample, "Symbol(\\"1\\")", "es3000") must return true'
  );
  assert.sameValue(sample[s1], "es3000", 'The value of sample[s1] is "es3000"');

  Object.defineProperty(sample, s2, {
    writable: false,
    value: undefined
  });
  assert.sameValue(
    Reflect.set(sample, s2, 42),
    false,
    'Reflect.set(sample, "Symbol(\\"2\\")", 42) must return false'
  );
  assert.sameValue(sample[s2], undefined, 'The value of sample[s2] is expected to equal `undefined`');
}, null, ["passthrough"]);
