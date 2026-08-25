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
includes: [propertyHelper.js]
---*/

var plainObject = Object.create(null);
Object.defineProperty(plainObject, "foo", {
  configurable: false,
});

var plainObjectTarget = new Proxy(plainObject, {});
var plainObjectProxy = new Proxy(plainObjectTarget, {
  defineProperty: null,
});

assert.throws(TypeError, function() {
  Object.defineProperty(plainObjectProxy, "foo", {
    configurable: true,
  });
});

Object.defineProperty(plainObjectProxy, "bar", {
  get: function() {
    return 2;
  },
});
assert.sameValue(plainObject.bar, 2);


var regExp = /(?:)/g;
var regExpTarget = new Proxy(regExp, {});
var regExpProxy = new Proxy(regExpTarget, {
  defineProperty: null,
});

assert(
  Reflect.defineProperty(regExpProxy, "lastIndex", {
    writable: false,
  })
);

verifyNotWritable(regExp, "lastIndex");
