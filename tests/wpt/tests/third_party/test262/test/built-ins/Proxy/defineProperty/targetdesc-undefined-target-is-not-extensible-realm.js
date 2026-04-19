// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-defineownproperty-p-desc
description: >
    Throw a TypeError exception if Desc is not configurable and target is not
    extensible, and trap result is true (honoring the realm of the current
    execution context).
info: |
    [[DefineOwnProperty]] (P, Desc)

    ...
    19. If targetDesc is undefined, then
        a. If extensibleTarget is false, throw a TypeError exception.
    ...
features: [cross-realm, Proxy]
---*/

var OProxy = $262.createRealm().global.Proxy;
var target = Object.create(null);
var p = new OProxy(target, {
  defineProperty: function() {
    return true;
  }
});

Object.preventExtensions(target);

assert.throws(TypeError, function() {
  p.prop = null;
});
