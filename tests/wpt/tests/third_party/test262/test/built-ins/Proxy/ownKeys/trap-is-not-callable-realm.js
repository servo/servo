// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
  Throws if trap is not callable (honoring the Realm of the current execution
  context)
info: |
    [[OwnPropertyKeys]] ( )

    5. Let trap be GetMethod(handler, "ownKeys").
    ...

    #sec-getmethod
    7.3.9 GetMethod (O, P)

    4. If IsCallable(func) is false, throw a TypeError exception.
features: [cross-realm, Proxy]
---*/

var OProxy = $262.createRealm().global.Proxy;
var p = new OProxy({
  attr: 1
}, {
  ownKeys: {}
});

assert.throws(TypeError, function() {
  Object.keys(p);
});
