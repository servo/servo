// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  If receiver is altered, `true` is returned for canonical numeric strings that are invalid indices.
  Value is not coerced.
info: |
  [[Set]] ( P, V, Receiver )

  [...]
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. If ! SameValue(O, Receiver) is true
        [...]
      ii. 1. Else if ! IsValidIntegerIndex(_O_, _numericIndex_) is *false*, return *true*.
includes: [testTypedArray.js]
features: [TypedArray, Reflect]
---*/

var valueOfCalls = 0;
var value = {
  valueOf: function() {
    ++valueOfCalls;
    return 2.3;
  },
};

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var target, receiver;

  [1, 1.5, -1].forEach(function(key) {
    Object.defineProperty(TA.prototype, key, {
      get: function() { throw new Test262Error(key + " getter should be unreachable!"); },
      set: function(_v) { throw new Test262Error(key + " setter should be unreachable!"); },
      configurable: true,
    });


    target = new TA(makeCtorArg([0]));
    receiver = {};
    assert(Reflect.set(target, key, value, receiver), "Reflect.set should succeed (key: " + key + ", receiver: empty object)");
    assert(!target.hasOwnProperty(key), "target[" + key + "] should not be created (receiver: empty object)");
    assert(!receiver.hasOwnProperty(key), "receiver[" + key + "] should not be created (receiver: empty object)");


    target = new TA(makeCtorArg([0]));
    receiver = new TA(makeCtorArg([1]));
    assert(Reflect.set(target, key, value, receiver), "Reflect.set should succeed (key: " + key + ", receiver: another typed array of the same length)");
    assert(!target.hasOwnProperty(key), "target[" + key + "] should not be created (receiver: another typed array of the same length)");
    assert(!receiver.hasOwnProperty(key), "receiver[" + key + "] should not be created (receiver: another typed array of the same length)");


    target = new TA(makeCtorArg([0]));
    receiver = Object.defineProperty({}, key, {
      get: function() { return 1; },
      set: function(_v) { throw new Test262Error(key + " setter should be unreachable!"); },
      configurable: true,
    });
    assert(Reflect.set(target, key, value, receiver), "Reflect.set should succeed (receiver: plain object with " + key + " accessor)");
    assert(!target.hasOwnProperty(key), "target[" + key + "] should not be created (receiver: plain object with " + key + " accessor)");
    assert.sameValue(receiver[key], 1, "receiver[" + key + "] should remain unchanged (receiver: plain object with " + key + " accessor)");


    target = new TA(makeCtorArg([0]));
    receiver = Object.defineProperty({}, key, { value: 1, writable: false, configurable: true });
    assert(Reflect.set(target, key, value, receiver), "Reflect.set should succeed (receiver: plain object with non-writable " + key + ")");
    assert(!target.hasOwnProperty(key), "target[" + key + "] should not be created (receiver: plain object with non-writable " + key + ")");
    assert.sameValue(receiver[key], 1, "receiver[" + key + "] should remain unchanged (receiver: plain object with non-writable " + key + ")");


    target = new TA(makeCtorArg([0]));
    receiver = Object.preventExtensions({});
    assert(Reflect.set(target, key, value, receiver), "Reflect.set should fail (key: " + key + ", receiver: non-extensible empty object)");
    assert(!target.hasOwnProperty(key), "target[" + key + "] should not be created (receiver: non-extensible empty object)");
    assert(!receiver.hasOwnProperty(key), "receiver[" + key + "] should not be created (receiver: non-extensible empty object)");


    assert(delete TA.prototype[key]);
  });


  target = new TA(makeCtorArg([0]));
  receiver = new TA(makeCtorArg([1, 1]));
  assert(Reflect.set(target, 1, value, receiver), "Reflect.set should succeed (receiver: another typed array of greater length)");
  assert(!target.hasOwnProperty(1), "target[1] should not be created (receiver: another typed array of greater length)");
  assert.sameValue(receiver[1], 1, "receiver[1] should remain unchanged (receiver: another typed array of greater length)");
}, null, ["passthrough"]);

assert.sameValue(valueOfCalls, 0, "value should not be coerced");
