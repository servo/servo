// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-preventextensions
description: >
  If "preventExtensions" trap is null or undefined, [[PreventExtensions]] call
  is properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[PreventExtensions]] ( )

  [...]
  4. Let target be O.[[ProxyTarget]].
  5. Let trap be ? GetMethod(handler, "preventExtensions").
  6. If trap is undefined, then
    a. Return ? target.[[PreventExtensions]]().
features: [Proxy]
---*/

var target = new Proxy({}, {
  preventExtensions: function(_target) {
    return false;
  },
});

var proxy = new Proxy(target, {});

assert.throws(TypeError, function() {
  Object.preventExtensions(proxy);
});
