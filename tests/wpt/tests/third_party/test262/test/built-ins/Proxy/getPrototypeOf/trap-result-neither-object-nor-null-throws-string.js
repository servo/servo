// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.1
description: >
    throw a TypeError exception if trap result is a String.
features: [Proxy]
---*/

var p = new Proxy({}, {
  getPrototypeOf: function() {
    return "";
  }
});

assert.throws(TypeError, function() {
  Object.getPrototypeOf(p);
});
