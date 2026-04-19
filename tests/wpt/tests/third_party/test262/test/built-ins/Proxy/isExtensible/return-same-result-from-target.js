// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.3
description: >
    Return trap result.
features: [Proxy]
---*/

var target = {};
var p = new Proxy(target, {
  isExtensible: function(t) {
    return Object.isExtensible(t);
  }
});

assert.sameValue(Object.isExtensible(p), true);

Object.preventExtensions(target);

assert.sameValue(Object.isExtensible(p), false);
