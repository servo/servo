// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.2
description: >
    Throws a TypeError exception if trap is not callable.
info: |
    [[SetPrototypeOf]] (V)

    ...
    2. Let handler be the value of the [[ProxyHandler]] internal slot of O.
    ...
    5. Let target be the value of the [[ProxyTarget]] internal slot of O.
    6. Let trap be GetMethod(handler, "setPrototypeOf").
    ...
        7.3.9 GetMethod (O, P)
        ...
        2. Let func be GetV(O, P).
        5. If IsCallable(func) is false, throw a TypeError exception.
        ...
features: [Proxy, Reflect, Reflect.setPrototypeOf]
---*/

var target = {};
var p = new Proxy(target, {
  setPrototypeOf: {}
});

assert.throws(TypeError, function() {
  Reflect.setPrototypeOf(p, {
    value: 1
  });
});
