// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.tostring
description: Proxy of an array is treated as an array
info: |
  [...]
  3. Let O be ToObject(this value).
  4. Let isArray be ? IsArray(O).
  5. If isArray is true, let builtinTag be "Array".
  [...]

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

assert.sameValue(
  Object.prototype.toString.call(objectProxy), '[object Object]'
);
assert.sameValue(
  Object.prototype.toString.call(arrayProxy), '[object Array]', 'array proxy'
);
assert.sameValue(
  Object.prototype.toString.call(arrayProxyProxy),
  '[object Array]',
  'proxy for array proxy'
);
