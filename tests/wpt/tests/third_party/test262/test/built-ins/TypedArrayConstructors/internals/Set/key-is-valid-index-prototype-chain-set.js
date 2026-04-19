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
features: [TypedArray, Proxy]
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

  Object.defineProperty(TA.prototype, 0, {
    get: function() { throw new Test262Error("0 getter should be unreachable!"); },
    set: function(_v) { throw new Test262Error("0 setter should be unreachable!"); },
    configurable: true,
  });


  target = new TA(makeCtorArg([0]));
  receiver = Object.create(target);
  receiver[0] = value;
  assert.sameValue(target[0], 0, "target[0] should remain unchanged (receiver: empty object)");
  assert.sameValue(receiver[0], value, "receiver[0] should be updated (receiver: empty object)");


  var proxyTrapCalls = 0;
  target = new TA(makeCtorArg([0]));
  receiver = new Proxy(Object.create(target), {
    defineProperty(_target, key, desc) {
      ++proxyTrapCalls;
      Object.defineProperty(_target, key, desc);
      return true;
    },
  });
  receiver[0] = value;
  assert.sameValue(target[0], 0, "target[0] should remain unchanged (receiver: proxy of an empty object)");
  assert.sameValue(receiver[0], value, "receiver[0] should be created (receiver: proxy of an empty object)");
  assert.sameValue(proxyTrapCalls, 1, "Proxy's [[DefineOwnProperty]] exotic method should be called");


  target = new TA(makeCtorArg([0]));
  receiver = Object.preventExtensions(Object.create(target));
  assert.throws(TypeError, function() { "use strict"; receiver[0] = value; },
    "setting receiver[0] should throw in strict mode (receiver: non-extensible empty object)");
  assert.sameValue(target[0], 0, "target[0] should remain unchanged (receiver: non-extensible empty object)");
  assert(!receiver.hasOwnProperty(0), "receiver[0] should not be created (receiver: non-extensible empty object)");


  target = new TA(makeCtorArg([0]));
  receiver = Object.setPrototypeOf([], target);
  receiver[0] = value;
  assert.sameValue(target[0], 0, "target[0] should remain unchanged (receiver: regular array)");
  assert.sameValue(receiver[0], value, "receiver[0] should be created (receiver: regular array)");
  assert.sameValue(receiver.length, 1, "Array's [[DefineOwnProperty]] exotic method should be called");


  target = new TA(makeCtorArg([0]));
  receiver = Object.setPrototypeOf(new String(""), target);
  receiver[0] = value;
  assert.sameValue(target[0], 0, "target[0] should remain unchanged (receiver: empty String object)");
  assert.sameValue(receiver[0], value, "receiver[0] should be created (receiver: empty String object)");


  assert(delete TA.prototype[0]);
}, null, ["passthrough"]);

assert.sameValue(valueOfCalls, 0, "value should not be coerced");
