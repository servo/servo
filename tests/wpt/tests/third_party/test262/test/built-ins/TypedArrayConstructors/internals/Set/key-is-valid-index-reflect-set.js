// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  If receiver is altered, OrdinarySet result is returned for valid indices.
  Value is not coerced.
info: |
  [[Set]] ( P, V, Receiver )

  [...]
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      [...]
  3. Return ? OrdinarySet(O, P, V, Receiver).
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
  function coerceValue(value) { return new TA(makeCtorArg([value]))[0]; }

  var target, receiver;

  Object.defineProperty(TA.prototype, 0, {
    get: function() { throw new Test262Error("0 getter should be unreachable!"); },
    set: function(_v) { throw new Test262Error("0 setter should be unreachable!"); },
    configurable: true,
  });


  target = new TA(makeCtorArg([0]));
  receiver = {};
  assert(Reflect.set(target, 0, value, receiver), "Reflect.set should succeed (receiver: empty object)");
  assert.sameValue(target[0], 0, "target[0] should remain unchanged (receiver: empty object)");
  assert.sameValue(receiver[0], value, "receiver[0] should be created (receiver: empty object)");


  target = new TA(makeCtorArg([0]));
  receiver = new TA(makeCtorArg([1]));
  assert(Reflect.set(target, 0, new Number(2.3), receiver), "Reflect.set should succeed (receiver: another typed array of the same length)");
  assert.sameValue(target[0], 0, "target[0] should remain unchanged (receiver: another typed array of the same length)");
  assert.sameValue(receiver[0], coerceValue(new Number(2.3)), "receiver[0] should be updated (receiver: another typed array of the same length)");


  target = new TA(makeCtorArg([0, 0]));
  receiver = new TA(makeCtorArg([1]));
  assert(!Reflect.set(target, 1, value, receiver), "Reflect.set should fail (receiver: another typed array of shorter length)");
  assert.sameValue(target[1], 0, "target[1] should remain unchanged (receiver: another typed array of shorter length)");
  assert(!receiver.hasOwnProperty(1), "receiver[1] should not be created (receiver: another typed array of shorter length)");


  target = new TA(makeCtorArg([0]));
  receiver = Object.preventExtensions({});
  assert(!Reflect.set(target, 0, value, receiver), "Reflect.set should fail (receiver: non-extensible empty object)");
  assert.sameValue(target[0], 0, "target[0] should remain unchanged (receiver: non-extensible empty object)");
  assert(!receiver.hasOwnProperty(0), "receiver[0] should not be created (receiver: non-extensible empty object)");


  target = new TA(makeCtorArg([0]));
  receiver = {
    get 0() { return 1; },
    set 0(_v) { throw new Test262Error("0 setter should be unreachable!"); },
  };
  assert(!Reflect.set(target, 0, value, receiver), "Reflect.set should fail (receiver: plain object with 0 accessor)");
  assert.sameValue(target[0], 0, "target[0] should remain unchanged (receiver: plain object with 0 accessor)");
  assert.sameValue(receiver[0], 1, "receiver[0] should remain unchanged (receiver: plain object with 0 accessor)");


  target = new TA(makeCtorArg([0]));
  receiver = Object.defineProperty({}, 0, { value: 1, writable: false, configurable: true });
  assert(!Reflect.set(target, 0, value, receiver), "Reflect.set should fail (receiver: plain object with non-writable 0)");
  assert.sameValue(target[0], 0, "target[0] should remain unchanged (receiver: plain object with non-writable 0)");
  assert.sameValue(receiver[0], 1, "receiver[0] should remain unchanged (receiver: plain object with non-writable 0)");


  assert(delete TA.prototype[0]);
}, null, ["passthrough"]);

assert.sameValue(valueOfCalls, 0, "value should not be coerced");
