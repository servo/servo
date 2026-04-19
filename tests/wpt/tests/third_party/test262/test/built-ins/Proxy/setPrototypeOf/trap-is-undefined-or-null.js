// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-setprototypeof-v
description: >
  Return target.[[SetPrototypeOf]] (V) if trap is undefined or null.
info: |
  [[SetPrototypeOf]] (V)

  6. Let trap be ? GetMethod(handler, "setPrototypeOf").
  7. If trap is undefined, then
    a. Return ? target.[[SetPrototypeOf]](V).

  GetMethod (V, P)

  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
features: [Proxy]
---*/

var proxy, called, value;
var target = new Proxy({}, {
  setPrototypeOf: function(t, v) {
    called += 1;
    value = v;
    return true;
  }
});
var proto = {};

proxy = new Proxy(target, {});
called = 0;
value = false;
Object.setPrototypeOf(proxy, proto);
assert.sameValue(called, 1, "undefined, target.[[SetPrototypeOf]] is called");
assert.sameValue(value, proto, "undefined, called with V");

proxy = new Proxy(target, {
  setPrototypeOf: null
});
called = 0;
value = false;
Object.setPrototypeOf(proxy, proto);
assert.sameValue(called, 1, "null, target.[[SetPrototypeOf]] is called");
assert.sameValue(value, proto, "null, called with V");
