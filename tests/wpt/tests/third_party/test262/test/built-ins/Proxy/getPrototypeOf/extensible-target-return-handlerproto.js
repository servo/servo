// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.1
description: >
    Return trap result if it's an Object and target is extensible.
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
    12. ReturnIfAbrupt(extensibleTarget).
    13. If extensibleTarget is true, return handlerProto.
    ...

features: [Proxy]
---*/

var prot = {
  foo: 1
};
var p = new Proxy({}, {
  getPrototypeOf: function() {
    return prot;
  }
});

assert.sameValue(Object.getPrototypeOf(p), prot);
