// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.filter
description: Abrupt completion from constructor that is a revoked Proxy object
info: |
    [...]
    5. Let A be ? ArraySpeciesCreate(O, 0).
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
var ctorCount = 0;
var cbCount = 0;
var cb = function() {
  cbCount += 1;
};

Object.defineProperty(o.proxy, 'constructor', {
  get: function() {
    ctorCount += 1;
  }
});
o.revoke();

assert.throws(TypeError, function() {
  Array.prototype.filter.call(o.proxy, cb);
});

assert.sameValue(ctorCount, 0, '`constructor` property not accessed');
assert.sameValue(cbCount, 0, 'callback not invoked');
