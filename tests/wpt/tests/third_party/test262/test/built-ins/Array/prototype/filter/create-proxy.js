// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.filter
description: Species constructor of a Proxy object whose target is an array
info: |
    [...]
    5. Let A be ? ArraySpeciesCreate(O, 0).
    [...]
    9. Return A.

    9.4.2.3 ArraySpeciesCreate

    [...]
    3. Let isArray be ? IsArray(originalArray).

    7.2.2 IsArray

    [...]
    3. If argument is a Proxy exotic object, then
       a. If the value of the [[ProxyHandler]] internal slot of argument is
          null, throw a TypeError exception.
       b. Let target be the value of the [[ProxyTarget]] internal slot of
          argument.
       c. Return ? IsArray(target).
features: [Proxy, Symbol.species]
---*/

var array = [];
var proxy = new Proxy(new Proxy(array, {}), {});
var Ctor = function() {};
var result;

array.constructor = function() {};
array.constructor[Symbol.species] = Ctor;

result = Array.prototype.filter.call(proxy, function() {});

assert.sameValue(Object.getPrototypeOf(result), Ctor.prototype);
