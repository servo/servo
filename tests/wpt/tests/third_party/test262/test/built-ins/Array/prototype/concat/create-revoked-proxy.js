// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.concat
description: Abrupt completion from constructor that is a revoked Proxy object
info: |
    1. Let O be ? ToObject(this value).
    2. Let A be ? ArraySpeciesCreate(O, 0).

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
  Array.prototype.concat.call(o.proxy);
}, 'Array.prototype.concat.call(o.proxy) throws a TypeError exception');

assert.sameValue(callCount, 0, 'The value of callCount is expected to be 0');
