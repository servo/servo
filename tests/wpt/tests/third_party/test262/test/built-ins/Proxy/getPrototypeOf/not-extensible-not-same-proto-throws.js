// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.1
description: >
    Throws a TypeError if the target is not extensible and the trap result is
    not the same as the target.[[GetPrototypeOf]] result.
info: |
    [[GetPrototypeOf]] ( )

    ...
    1. Let handler be the value of the [[ProxyHandler]] internal slot of O.
    ...
    4. Let target be the value of the [[ProxyTarget]] internal slot of O.
    5. Let trap be GetMethod(handler, "getPrototypeOf").
    ...
    8. Let handlerProto be Call(trap, handler, «target»).
    ...
    11. Let extensibleTarget be IsExtensible(target).
    ...
    14. Let targetProto be target.[[GetPrototypeOf]]().
    15. ReturnIfAbrupt(targetProto).
    16. If SameValue(handlerProto, targetProto) is false, throw a TypeError
    exception.
    ...
features: [Proxy]
---*/

var target = Object.create({
  foo: 1
});

var p = new Proxy(target, {
  getPrototypeOf: function() {
    return {};
  }
});

Object.preventExtensions(target);

assert.throws(TypeError, function() {
  Object.getPrototypeOf(p);
});
