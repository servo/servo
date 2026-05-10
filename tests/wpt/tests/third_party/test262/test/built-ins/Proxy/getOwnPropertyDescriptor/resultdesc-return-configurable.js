// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.5
description: >
    Return descriptor from trap result if it has the same value as the target
    property descriptor.
features: [Proxy]
---*/

var target = {};
var descriptor = {
  configurable: true,
  enumerable: true,
  value: 1
};

Object.defineProperty(target, "bar", descriptor);

var p = new Proxy(target, {
  getOwnPropertyDescriptor: function(t, prop) {
    return Object.getOwnPropertyDescriptor(t, prop);
  }
});

var proxyDesc = Object.getOwnPropertyDescriptor(p, "bar");

assert(proxyDesc.configurable);
assert(proxyDesc.enumerable);
assert.sameValue(proxyDesc.value, 1);
assert.sameValue(proxyDesc.writable, false);
