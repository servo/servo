// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-getownproperty-p
description: >
  Error when trap result is neither Object nor undefined (honoring the Realm of
  the current execution context)
info: |
  [...]
  9. If Type(trapResultObj) is neither Object nor Undefined, throw a TypeError
     exception.
features: [cross-realm, Proxy]
---*/

var OProxy = $262.createRealm().global.Proxy;

var p = new OProxy({}, {
  getOwnPropertyDescriptor: function() {
    return null;
  }
});

assert.throws(TypeError, function() {
  Object.getOwnPropertyDescriptor(p, 'x');
});
