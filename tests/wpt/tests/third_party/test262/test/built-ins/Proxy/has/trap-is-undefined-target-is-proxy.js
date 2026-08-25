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
features: [Proxy, Reflect]
---*/

var plainObject = {
  get 0() {
    return 1;
  },
  foo: 2,
  set bar(_value) {},
};

var plainObjectTarget = new Proxy(plainObject, {});
var plainObjectProxy = new Proxy(plainObjectTarget, {
  get: undefined,
});

assert(0 in Object.create(plainObjectProxy));
assert("foo" in plainObjectProxy);
assert(Reflect.has(plainObjectProxy, "bar"));


var arrayTarget = new Proxy([1, 2], {});
var arrayProxy = new Proxy(arrayTarget, {
  get: undefined,
});

assert("length" in Object.create(arrayProxy));
assert("1" in arrayProxy);
assert(!("2" in arrayProxy));
