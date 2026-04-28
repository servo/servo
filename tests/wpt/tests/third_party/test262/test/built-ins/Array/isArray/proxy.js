// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.isarray
es6id: 22.1.2.2
description: Proxy of an array is treated as an array
info: |
  1. Return IsArray(arg).

  7.2.2 IsArray

  [...]
  3. If argument is a Proxy exotic object, then
     a. If the value of the [[ProxyHandler]] internal slot of argument is null,
        throw a TypeError exception.
     b. Let target be the value of the [[ProxyTarget]] internal slot of
        argument.
     c. Return ? IsArray(target).
features: [Proxy]
---*/

var objectProxy = new Proxy({}, {});
var arrayProxy = new Proxy([], {});
var arrayProxyProxy = new Proxy(arrayProxy, {});

assert.sameValue(Array.isArray(objectProxy), false, 'Array.isArray(new Proxy({}, {})) must return false');
assert.sameValue(Array.isArray(arrayProxy), true, 'Array.isArray(new Proxy([], {})) must return true');
assert.sameValue(
  Array.isArray(arrayProxyProxy), true, 'Array.isArray(new Proxy(arrayProxy, {})) must return true'
);
