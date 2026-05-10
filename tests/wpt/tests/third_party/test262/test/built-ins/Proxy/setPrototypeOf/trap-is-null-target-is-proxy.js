// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-setprototypeof-v
description: >
  If "setPrototypeOf" trap is null or undefined, [[SetPrototypeOf]] call
  is properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[SetPrototypeOf]] ( V )

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "setPrototypeOf").
  7. If trap is undefined, then
    a. Return ? target.[[SetPrototypeOf]](V).

  OrdinarySetPrototypeOf ( O, V )

  [...]
  8. Repeat, while done is false,
    a. If p is null, set done to true.
    b. Else if SameValue(p, O) is true, return false.
    [...]
features: [Proxy]
---*/

var plainObject = {};
var plainObjectTarget = new Proxy(plainObject, {});
var plainObjectProxy = new Proxy(plainObjectTarget, {
  setPrototypeOf: null,
});

Object.setPrototypeOf(plainObjectProxy, null);
assert.sameValue(Object.getPrototypeOf(plainObject), null);

var cyclicPrototype = Object.create(plainObject);
assert.throws(TypeError, function() {
  Object.setPrototypeOf(plainObjectProxy, cyclicPrototype);
});
