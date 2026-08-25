// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
  If "construct" trap is null or undefined, [[Construct]] call is
  properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[Construct]] ( argumentsList, newTarget )

  [...]
  4. Let target be O.[[ProxyTarget]].
  5. Assert: IsConstructor(target) is true.
  6. Let trap be ? GetMethod(handler, "construct").
  7. If trap is undefined, then
    a. Return ? Construct(target, argumentsList, newTarget).
features: [class, Proxy, Reflect, Reflect.construct]
includes: [compareArray.js]
---*/

var ArrayTarget = new Proxy(Array, {});
var ArrayProxy = new Proxy(ArrayTarget, {});

var array = new ArrayProxy(1, 2, 3);
assert(Array.isArray(array));
assert.compareArray(array, [1, 2, 3]);

class MyArray extends Array {
  get isMyArray() {
    return true;
  }
}

var myArray = Reflect.construct(ArrayProxy, [], MyArray);
assert(Array.isArray(myArray));
assert(myArray instanceof MyArray);
assert(myArray.isMyArray);
