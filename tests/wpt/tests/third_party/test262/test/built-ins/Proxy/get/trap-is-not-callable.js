// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.8
description: >
    Trap is not callable.
info: |
    [[Get]] (P, Receiver)

    6. Let trap be GetMethod(handler, "get").
    ...

    7.3.9 GetMethod (O, P)

    5. If IsCallable(func) is false, throw a TypeError exception.
features: [Proxy]
---*/

var p = new Proxy({}, {
  get: {}
});

assert.throws(TypeError, function() {
  p.attr;
});

assert.throws(TypeError, function() {
  p["attr"];
});
