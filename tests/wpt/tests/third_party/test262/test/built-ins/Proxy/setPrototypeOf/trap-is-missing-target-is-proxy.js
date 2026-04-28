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

  [[SetPrototypeOf]] ( V )

  [...]
  8. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target, V »)).
  9. If booleanTrapResult is false, return false.
features: [Proxy]
---*/

var target = new Proxy({}, {
  setPrototypeOf: function(_target, _value) {
    return false;
  },
});

var proxy = new Proxy(target, {});

assert.throws(TypeError, function() {
  Object.setPrototypeOf(proxy, null);
});
