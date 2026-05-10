// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.4
description: >
    If boolean trap result if false, return false.
features: [Proxy, Reflect]
---*/

var target = {};
var p = new Proxy({}, {
  preventExtensions: function(t) {
    return 0;
  }
});

assert.sameValue(Reflect.preventExtensions(p), false);

Object.preventExtensions(target);

assert.sameValue(Reflect.preventExtensions(p), false);
