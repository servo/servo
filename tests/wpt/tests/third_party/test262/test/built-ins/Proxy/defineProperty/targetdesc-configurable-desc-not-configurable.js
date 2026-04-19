// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.6
description: >
    Throw a TypeError exception if Desc is not configurable and target property
    descriptor is configurable and trap result is true.
info: |
    [[DefineOwnProperty]] (P, Desc)

    ...
    20. Else targetDesc is not undefined,
        b. If settingConfigFalse is true and targetDesc.[[Configurable]] is
        true, throw a TypeError exception.
    ...
features: [Proxy]
---*/

var target = {};
var p = new Proxy(target, {
  defineProperty: function(t, prop, desc) {
    return true;
  }
});

Object.defineProperty(target, "foo", {
  value: 1,
  configurable: true
});

assert.throws(TypeError, function() {
  Object.defineProperty(p, "foo", {
    value: 1,
    configurable: false
  });
});
