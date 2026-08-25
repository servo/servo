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

var stringTarget = new Proxy(new String("str"), {});
var stringProxy = new Proxy(stringTarget, {});

verifyProperty(stringProxy, "0", {
  value: "s",
  writable: false,
  enumerable: true,
  configurable: false,
});

verifyProperty(stringProxy, "length", {
  value: 3,
  writable: false,
  enumerable: false,
  configurable: false,
});


var functionTarget = new Proxy(function() {}, {});
var functionProxy = new Proxy(functionTarget, {});

verifyProperty(functionProxy, "prototype", {
  writable: true,
  enumerable: false,
  configurable: false,
});
