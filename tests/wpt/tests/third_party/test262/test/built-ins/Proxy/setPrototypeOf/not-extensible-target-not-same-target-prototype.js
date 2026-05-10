// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.2
description: >
    Throws a TypeError exception if boolean trap result is true, target is
    not extensible, and the given parameter is not the same object as the target
    prototype.
info: |
    [[SetPrototypeOf]] (V)

    ...
    2. Let handler be the value of the [[ProxyHandler]] internal slot of O.
    ...
    5. Let target be the value of the [[ProxyTarget]] internal slot of O.
    6. Let trap be GetMethod(handler, "setPrototypeOf").
    ...
    9. Let booleanTrapResult be ToBoolean(Call(trap, handler, «target, V»)).
    14. Let targetProto be target.[[GetPrototypeOf]]().
    15. ReturnIfAbrupt(targetProto).
    16. If booleanTrapResult is true and SameValue(V, targetProto) is false,
    throw a TypeError exception.
    ...
features: [Proxy, Reflect, Reflect.setPrototypeOf]
---*/

var target, proxy;
target = {};
proxy = new Proxy(target, {
  setPrototypeOf: function() {
    return true;
  }
});

Object.preventExtensions(target);

assert.throws(TypeError, function() {
  Reflect.setPrototypeOf(proxy, {});
}, "target prototype is different");

var proto = {};
target = Object.setPrototypeOf({}, proto);
proxy = new Proxy(target, {
  setPrototypeOf: function() {
    Object.setPrototypeOf(target, {});
    Object.preventExtensions(target);
    return true;
  }
});

assert.throws(TypeError, function() {
  Reflect.setPrototypeOf(proxy, proto);
}, "target prototype is changed inside trap handler");
