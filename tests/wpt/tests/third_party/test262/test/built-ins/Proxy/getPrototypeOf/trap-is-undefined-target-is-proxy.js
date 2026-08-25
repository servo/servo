// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-getprototypeof
description: >
  If "getPrototypeOf" trap is null or undefined, [[GetPrototypeOf]] call
  is properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[GetPrototypeOf]] ( )

  [...]
  4. Let target be O.[[ProxyTarget]].
  5. Let trap be ? GetMethod(handler, "getPrototypeOf").
  6. If trap is undefined, then
    a. Return ? target.[[GetPrototypeOf]]()
features: [Proxy]
---*/

var arrayTarget = new Proxy([], {});
var arrayProxy = new Proxy(arrayTarget, {
  getPrototypeOf: undefined,
});

assert.sameValue(Object.getPrototypeOf(arrayProxy), Array.prototype);
