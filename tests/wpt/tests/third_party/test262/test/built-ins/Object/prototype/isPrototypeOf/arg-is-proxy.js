// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.isprototypeof
description: >
  If V is a Proxy object, only its "getPrototypeOf" trap gets invoked.
info: |
  Object.prototype.isPrototypeOf ( V )

  ...
  3. Repeat,
    a. Set V to ? V.[[GetPrototypeOf]]().
    b. If V is null, return false.
    c. If SameValue(O, V) is true, return true.
includes: [proxyTrapsHelper.js]
features: [Proxy]
---*/

var proxyProto = [];
var handler = allowProxyTraps({
  getPrototypeOf: function(_target) {
    return proxyProto;
  },
});

var proxy = new Proxy({}, handler);

assert.sameValue(proxyProto.isPrototypeOf(proxy), true);
