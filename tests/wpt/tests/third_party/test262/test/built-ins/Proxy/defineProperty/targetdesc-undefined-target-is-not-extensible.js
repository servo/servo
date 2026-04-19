// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.6
description: >
    Throw a TypeError exception if Desc is not configurable and target is not
    extensible, and trap result is true.
info: |
    [[DefineOwnProperty]] (P, Desc)

    ...
    19. If targetDesc is undefined, then
        a. If extensibleTarget is false, throw a TypeError exception.
    ...
features: [Proxy]
---*/

var target = {};
var p = new Proxy(target, {
  defineProperty: function(t, prop, desc) {
    return true;
  }
});

Object.preventExtensions(target);

assert.throws(TypeError, function() {
  Object.defineProperty(p, "foo", {});
});
