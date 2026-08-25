// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-getownproperty-p
description: >
  If "getOwnPropertyDescriptor" trap is null or undefined, [[GetOwnProperty]]
  call is properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[GetOwnProperty]] ( P )

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "getOwnPropertyDescriptor").
  7. If trap is undefined, then
    a. Return ? target.[[GetOwnProperty]](P).
includes: [propertyHelper.js]
features: [Proxy]
---*/

var plainObjectTarget = new Proxy({foo: 1}, {});
var plainObjectProxy = new Proxy(plainObjectTarget, {
  getOwnPropertyDescriptor: null,
});

verifyProperty(plainObjectProxy, "bar", undefined);
verifyProperty(plainObjectProxy, "foo", {
  value: 1,
  writable: true,
  enumerable: true,
  configurable: true,
});


var fooDescriptor = {
  get: function() {},
  set: function(_value) {},
  enumerable: false,
  configurable: true,
};

var target = new Proxy({}, {
  getOwnPropertyDescriptor: function(_target, key) {
    if (key === "foo") {
      return fooDescriptor;
    }
  },
  deleteProperty: function(_target, key) {
    if (key === "foo") {
      fooDescriptor = undefined;
    }

    return true;
  },
});

var proxy = new Proxy(target, {
  getOwnPropertyDescriptor: null,
});

verifyProperty(proxy, "bar", undefined);
verifyProperty(proxy, "foo", fooDescriptor);
