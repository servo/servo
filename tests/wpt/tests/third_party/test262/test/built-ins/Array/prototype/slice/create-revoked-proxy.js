// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.slice
description: Abrupt completion from constructor that is a revoked Proxy object
info: |
    [...]
    8. Let A be ? ArraySpeciesCreate(O, count).
    [...]

    9.4.2.3 ArraySpeciesCreate

    [...]
    3. Let isArray be ? IsArray(originalArray).

    7.2.2 IsArray

    [...]
    3. If argument is a Proxy exotic object, then
       a. If the value of the [[ProxyHandler]] internal slot of argument is
          null, throw a TypeError exception.
features: [Proxy]
---*/

var o = Proxy.revocable([], {});
var callCount = 0;

Object.defineProperty(o.proxy, 'constructor', {
  get: function() {
    callCount += 1;
  }
});
o.revoke();

assert.throws(TypeError, function() {
  Array.prototype.slice.call(o.proxy);
});

assert.sameValue(callCount, 0, '`constructor` property not accessed');
