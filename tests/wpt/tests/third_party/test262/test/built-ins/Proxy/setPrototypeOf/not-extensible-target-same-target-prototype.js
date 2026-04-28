// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-setprototypeof-v
description: >
  Handler can only return true for non-extensible targets if the given prototype
  is the same as target's prototype
info: |
  [[SetPrototypeOf]] (V)

  12. Let targetProto be ? target.[[GetPrototypeOf]]().
  13. If SameValue(V, targetProto) is false, throw a TypeError exception.
  14. Return true.
features: [Proxy, Reflect, Reflect.setPrototypeOf]
---*/

var proto = {};
var target = Object.setPrototypeOf({}, proto);

Object.preventExtensions(target);

var proxy;

proxy = new Proxy(target, {
  setPrototypeOf: function() {
    return true;
  }
});

assert.sameValue(
  Reflect.setPrototypeOf(proxy, proto),
  true,
  "prototype arg is the same in target"
);

var outro = {};
proxy = new Proxy(outro, {
  setPrototypeOf: function(t, p) {
    Object.setPrototypeOf(t, p);
    Object.preventExtensions(t);
    return true;
  }
});

assert.sameValue(
  Reflect.setPrototypeOf(proxy, proto),
  true,
  "prototype is set to target inside handler trap"
);
assert.sameValue(
  Object.getPrototypeOf(outro),
  proto,
  "target has the custom set prototype"
);
