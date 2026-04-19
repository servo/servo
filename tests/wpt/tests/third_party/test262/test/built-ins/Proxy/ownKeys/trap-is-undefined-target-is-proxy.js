// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
  If "ownKeys" trap is null or undefined, [[OwnPropertyKeys]] call is
  properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[OwnPropertyKeys]] ( )

  [...]
  4. Let target be O.[[ProxyTarget]].
  5. Let trap be ? GetMethod(handler, "ownKeys").
  6. If trap is undefined, then
    a. Return ? target.[[OwnPropertyKeys]]().

  [[OwnPropertyKeys]] ( )

  [...]
  7. Let trapResultArray be ? Call(trap, handler, « target »).
  8. Let trapResult be ? CreateListFromArrayLike(trapResultArray, « String, Symbol »).
  [...]
  23. Return trapResult.
includes: [compareArray.js]
features: [Symbol, Proxy, Reflect]
---*/

var trapResult = [Symbol(), "length", "foo", "0"];
var target = new Proxy([], {
  ownKeys: function(_target) {
    return trapResult;
  },
});

var proxy = new Proxy(target, {
  ownKeys: undefined,
});

assert.compareArray(Reflect.ownKeys(proxy), trapResult);
