// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.5
description: >
    Throws a TypeError exception if trap result is not configurable but target
    property descriptor is undefined.
info: |
    [[GetOwnProperty]] (P)

    ...
    2. Let handler be the value of the [[ProxyHandler]] internal slot of O.
    ...
    5. Let target be the value of the [[ProxyTarget]] internal slot of O.
    6. Let trap be GetMethod(handler, "getOwnPropertyDescriptor").
    ...
    9. Let trapResultObj be Call(trap, handler, «target, P»).
    ...
    12. Let targetDesc be target.[[GetOwnProperty]](P).
    ...
    17. Let resultDesc be ToPropertyDescriptor(trapResultObj).
    ...
    22. If resultDesc.[[Configurable]] is false, then
        a. If targetDesc is undefined or targetDesc.[[Configurable]] is true, then
            i. Throw a TypeError exception.

features: [Proxy]
---*/

var target = {};

var p = new Proxy(target, {
  getOwnPropertyDescriptor: function(t, prop) {
    var foo = {};

    Object.defineProperty(foo, "bar", {
      configurable: false,
      enumerable: true,
      value: 1
    });

    return Object.getOwnPropertyDescriptor(foo, prop);
  }
});

assert.throws(TypeError, function() {
  Object.getOwnPropertyDescriptor(p, "bar");
});
