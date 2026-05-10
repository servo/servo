// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-defineownproperty-p-desc
description: >
    Throw a TypeError exception if Desc and target property descriptor are not
    compatible and trap result is true.
info: |
    [[DefineOwnProperty]] (P, Desc)

    ...
    20. Else targetDesc is not undefined,
        a. If IsCompatiblePropertyDescriptor(extensibleTarget, Desc ,
        targetDesc) is false, throw a TypeError exception.
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

Object.defineProperty(target, 'prop', {
  value: 1,
  configurable: false
});

assert.throws(TypeError, function() {
  Object.defineProperty(p, 'prop', {
    value: 1,
    configurable: true
  });
});
