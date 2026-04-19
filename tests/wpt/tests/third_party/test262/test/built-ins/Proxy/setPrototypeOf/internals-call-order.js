// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-setprototypeof-v
description: >
  Calls target.[[GetPrototypeOf]] after trap result as false and not extensible
  target
info: |
  [[SetPrototypeOf]] (V)

  8. Let booleanTrapResult be ToBoolean(? Call(trap, handler, « target, V »)).
  9. If booleanTrapResult is false, return false.
  10. Let extensibleTarget be ? IsExtensible(target).
  11. If extensibleTarget is true, return true.
  12. Let targetProto be ? target.[[GetPrototypeOf]]().
features: [Proxy, Reflect, Reflect.setPrototypeOf]
---*/

var calls = [];
var proto = {};

var target = new Proxy(Object.create(proto), {
  isExtensible: function() {
    calls.push("target.[[IsExtensible]]");
    return false;
  },
  getPrototypeOf: function() {
    calls.push("target.[[GetPrototypeOf]]");
    return proto;
  }
});

// Proxy must report same extensiblitity as target
Object.preventExtensions(target);

var proxy = new Proxy(target, {
  setPrototypeOf: function() {
    calls.push("proxy.[[setPrototypeOf]]");
    return true;
  }
});

assert.sameValue(Reflect.setPrototypeOf(proxy, proto), true);
assert.sameValue(calls.length, 3);
assert.sameValue(calls[0], "proxy.[[setPrototypeOf]]");
assert.sameValue(calls[1], "target.[[IsExtensible]]");
assert.sameValue(calls[2], "target.[[GetPrototypeOf]]");
