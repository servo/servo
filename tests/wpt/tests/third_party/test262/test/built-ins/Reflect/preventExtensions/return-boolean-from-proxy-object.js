// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.12
description: >
  Returns boolean from Proxy object.
info: |
  26.1.12 Reflect.preventExtensions ( target )

  ...
  2. Return target.[[PreventExtensions]]().

  9.5.4 [[PreventExtensions]] ( )

  8. Let booleanTrapResult be ToBoolean(Call(trap, handler, «target»)).
  9. ReturnIfAbrupt(booleanTrapResult).
  10. If booleanTrapResult is true, then
    a. Let targetIsExtensible be target.[[IsExtensible]]().
    b. ReturnIfAbrupt(targetIsExtensible).
    c. If targetIsExtensible is true, throw a TypeError exception.
  11. Return booleanTrapResult.
features: [Proxy, Reflect]
---*/

var p1 = new Proxy({}, {
  preventExtensions: function() {
    return false;
  }
});

assert.sameValue(
  Reflect.preventExtensions(p1), false,
  'returns false from Proxy handler'
);

var p2 = new Proxy({}, {
  preventExtensions: function(target) {
    Object.preventExtensions(target);
    return true;
  }
});

assert.sameValue(
  Reflect.preventExtensions(p2), true,
  'returns true from Proxy handler'
);
