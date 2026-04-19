// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-get-p-receiver
description: >
  If "get" trap is null or undefined, [[Get]] call is properly
  forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[Get]] ( P, Receiver )

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "get").
  7. If trap is undefined, then
    a. Return ? target.[[Get]](P, Receiver).
features: [Proxy]
includes: [compareArray.js]
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

assert.sameValue(Object.create(plainObjectProxy)[0], 1);
assert.sameValue(plainObjectProxy.foo, 2);
assert.sameValue(plainObjectProxy.bar, undefined);


var array = [1, 2, 3];
var arrayTarget = new Proxy(array, {});
var arrayProxy = new Proxy(arrayTarget, {
  get: undefined,
});

assert.compareArray(arrayProxy, array);
