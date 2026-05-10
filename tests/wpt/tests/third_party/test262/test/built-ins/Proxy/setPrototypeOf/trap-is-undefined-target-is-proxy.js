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
  4. Let extensible be O.[[Extensible]].
  5. If extensible is false, return false.
features: [Proxy]
---*/

var array = [];
var arrayTarget = new Proxy(array, {});
var arrayProxy = new Proxy(arrayTarget, {
  setPrototypeOf: undefined,
});

Object.setPrototypeOf(arrayProxy, Number.prototype);
assert.sameValue(Object.getPrototypeOf(array), Number.prototype);

Object.preventExtensions(array);
assert.throws(TypeError, function() {
  Object.setPrototypeOf(arrayProxy, null);
});
