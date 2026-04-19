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

var arrayTarget = new Proxy([42], {});
var arrayProxy = new Proxy(arrayTarget, {
  getOwnPropertyDescriptor: undefined,
});

verifyProperty(arrayProxy, "0", {
  value: 42,
  writable: true,
  enumerable: true,
  configurable: true,
});

verifyProperty(arrayProxy, "length", {
  value: 1,
  // writable: true,
  enumerable: false,
  configurable: false,
});


var regExpTarget = new Proxy(/(?:)/, {});
var regExpProxy = new Proxy(regExpTarget, {
  getOwnPropertyDescriptor: undefined,
});

verifyProperty(regExpProxy, "lastIndex", {
  value: 0,
  writable: true,
  enumerable: false,
  configurable: false,
});
