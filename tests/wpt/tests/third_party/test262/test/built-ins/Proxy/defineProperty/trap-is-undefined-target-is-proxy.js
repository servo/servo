// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-defineownproperty-p-desc
description: >
  If "defineProperty" trap is null or undefined, [[DefineOwnProperty]] call
  is properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[DefineOwnProperty]] (P, Desc)

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "defineProperty").
  7. If trap is undefined, then
    a. Return ? target.[[DefineOwnProperty]](P, Desc).
features: [Proxy, Reflect]
includes: [compareArray.js]
---*/

var array = [];
var arrayTarget = new Proxy(array, {});
var arrayProxy = new Proxy(arrayTarget, {
  defineProperty: undefined,
});

Object.defineProperty(arrayProxy, "0", {value: 1});
assert.compareArray(array, [1]);

assert.throws(TypeError, function() {
  Object.defineProperty(arrayProxy, "length", {
    get: function() {},
  });
});


var trapCalls = 0;
var target = new Proxy({}, {
  defineProperty: function(_target, key) {
    trapCalls++;
    return key === "foo";
  },
});

var proxy = new Proxy(target, {
  defineProperty: undefined,
});

assert(Reflect.defineProperty(proxy, "foo", {}));
assert.sameValue(trapCalls, 1);

assert.throws(TypeError, function() {
  Object.defineProperty(proxy, "bar", {});
});
assert.sameValue(trapCalls, 2);
