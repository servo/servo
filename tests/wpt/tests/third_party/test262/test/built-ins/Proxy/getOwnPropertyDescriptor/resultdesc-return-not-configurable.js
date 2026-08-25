// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.5
description: >
    Return descriptor from trap result if it has the same value as the target
    property descriptor and they're not configurable.
features: [Proxy]
---*/

var target = {};

Object.defineProperty(target, "attr", {
  configurable: false,
  enumerable: true,
  value: 1
});

var p = new Proxy(target, {
  getOwnPropertyDescriptor: function(t, prop) {
    return Object.getOwnPropertyDescriptor(t, prop);
  }
});

var proxyDesc = Object.getOwnPropertyDescriptor(p, "attr");

assert.sameValue(proxyDesc.configurable, false);
assert(proxyDesc.enumerable);
assert.sameValue(proxyDesc.value, 1);
assert.sameValue(proxyDesc.writable, false);
