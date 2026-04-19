// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-preventextensions
description: >
  Throws if trap is not callable (honoring the Realm of the current execution
  context)
info: |
    [[PreventExtensions]] ( )

    ...
    1. Let handler be the value of the [[ProxyHandler]] internal slot of O.
    ...
    5. Let trap be GetMethod(handler, "preventExtensions").
    ...
        7.3.9 GetMethod (O, P)
        ...
        2. Let func be GetV(O, P).
        5. If IsCallable(func) is false, throw a TypeError exception.
        ...
features: [cross-realm, Proxy]
---*/

var OProxy = $262.createRealm().global.Proxy;
var p = new OProxy({}, {
  preventExtensions: {}
});

assert.throws(TypeError, function() {
  Object.preventExtensions(p);
});
