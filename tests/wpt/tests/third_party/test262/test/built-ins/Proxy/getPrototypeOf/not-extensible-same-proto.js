// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.1
description: >
    Return trap result is target is not extensible, but trap result has the same
    value as target.[[GetPrototypeOf]] result.
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
    ...
    17. Return handlerProto.

features: [Proxy]
---*/

var target = Object.create(Array.prototype);

var p = new Proxy(target, {
  getPrototypeOf: function() {
    return Array.prototype;
  }
});

Object.preventExtensions(target);

assert.sameValue(Object.getPrototypeOf(p), Array.prototype);
