// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-hasproperty-p
description: >
  If "has" trap is null or undefined, [[HasProperty]] call is properly
  forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[HasProperty]] ( P )

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "has").
  7. If trap is undefined, then
    a. Return ? target.[[HasProperty]](P).
features: [Proxy, Symbol, Reflect, Array.prototype.includes]
---*/

var stringTarget = new Proxy(new String("str"), {});
var stringProxy = new Proxy(stringTarget, {
  get: null,
});

assert(Reflect.has(stringProxy, "length"));
assert(0 in stringProxy);
assert(!(4 in stringProxy));


var sym = Symbol();
var target = new Proxy({}, {
  has: function(_target, key) {
    return [sym, "6", "foo"].includes(key);
  },
});

var proxy = new Proxy(target, {
  get: null,
});

assert(Reflect.has(proxy, sym));
assert("6" in proxy);
assert("foo" in Object.create(proxy));
assert(!("bar" in proxy));
