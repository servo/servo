// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.5
description: >
    Throws a TypeError exception if trap result is undefined and target property
    descriptor is not configurable
info: |
    [[GetOwnProperty]] (P)

    ...
    14. If trapResultObj is undefined, then
        ...
        b. If targetDesc.[[Configurable]] is false, throw a TypeError exception.
    ...
features: [Proxy]
---*/

var target = {};
Object.defineProperty(target, "foo", {
  configurable: false,
  enumerable: false,
  value: 1
});

var p = new Proxy(target, {
  getOwnPropertyDescriptor: function(t, prop) {
    return;
  }
});

assert.throws(TypeError, function() {
  Object.getOwnPropertyDescriptor(p, "foo");
});
