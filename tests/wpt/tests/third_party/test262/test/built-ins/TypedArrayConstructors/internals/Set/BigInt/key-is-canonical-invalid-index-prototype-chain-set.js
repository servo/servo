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
features: [BigInt, TypedArray, Proxy]
---*/

var valueOfCalls = 0;
var value = {
  valueOf: function() {
    ++valueOfCalls;
    return 2n;
  },
};

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var target, receiver;

  [1, 1.5, -1].forEach(function(key) {
    Object.defineProperty(TA.prototype, key, {
      get: function() { throw new Test262Error(key + " getter should be unreachable!"); },
      set: function(_v) { throw new Test262Error(key + " setter should be unreachable!"); },
      configurable: true,
    });


    target = new TA(makeCtorArg([0n]));
    receiver = Object.create(target);
    receiver[key] = value;
    assert(!target.hasOwnProperty(key), "target[" + key + "] should not be created (receiver: empty object)");
    assert(!receiver.hasOwnProperty(key), "receiver[" + key + "] should not be created (receiver: empty object)");


    var proxyTrapCalls = 0;
    target = new TA(makeCtorArg([0n]));
    receiver = new Proxy(Object.create(target), {
      defineProperty(_target, key, desc) {
        ++proxyTrapCalls;
        Object.defineProperty(_target, key, desc);
        return true;
      },
    });
    receiver[key] = value;
    assert(!target.hasOwnProperty(key), "target[" + key + "] should not be created (receiver: proxy of an empty object)");
    assert(!receiver.hasOwnProperty(key), "receiver[" + key + "] should not be created (receiver: proxy of an empty object)");
    assert.sameValue(proxyTrapCalls, 0, "Proxy's [[DefineOwnProperty]] exotic method should not be called (key: " + key + ")");


    target = new TA(makeCtorArg([0n]));
    receiver = Object.preventExtensions(Object.create(target));
    receiver[key] = value;
    assert(!target.hasOwnProperty(key), "target[" + key + "] should not be created (receiver: non-extensible empty object)");
    assert(!receiver.hasOwnProperty(key), "receiver[" + key + "] should not be created (receiver: non-extensible empty object)");


    assert(delete TA.prototype[key]);
  });


  target = new TA(makeCtorArg([0n]));
  receiver = Object.setPrototypeOf([], target);
  receiver[1] = value;
  assert(!target.hasOwnProperty(1), "target[1] should not be created (receiver: regular array)");
  assert(!receiver.hasOwnProperty(1), "receiver[1] should not be created (receiver: regular array)");
  assert.sameValue(receiver.length, 0, "Array's [[DefineOwnProperty]] exotic method should not be called");


  target = new TA(makeCtorArg([0n]));
  receiver = Object.setPrototypeOf(new String(""), target);
  receiver[1] = value;
  assert(!target.hasOwnProperty(1), "target[1] should not be created (receiver: empty String object)");
  assert(!receiver.hasOwnProperty(1), "receiver[1] should remain unchanged (receiver: empty String object)");
}, null, ["passthrough"]);

assert.sameValue(valueOfCalls, 0, "value should not be coerced");
