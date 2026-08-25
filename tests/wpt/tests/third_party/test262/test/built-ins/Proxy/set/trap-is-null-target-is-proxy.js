// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-set-p-v-receiver
description: >
  If "set" trap is null or undefined, [[Set]] call is properly
  forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[Set]] ( P, V, Receiver )

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "set").
  7. If trap is undefined, then
    a. Return ? target.[[Set]](P, V, Receiver).
features: [Proxy, Reflect]
includes: [compareArray.js]
---*/

var array = [1, 2, 3];
var arrayTarget = new Proxy(array, {});
var arrayProxy = new Proxy(arrayTarget, {
  set: null,
});

arrayProxy.length = 0;
assert.compareArray(array, []);

Object.preventExtensions(array);

assert(!Reflect.set(arrayProxy, "foo", 2));
assert.throws(TypeError, function() {
  "use strict";
  arrayProxy[0] = 3;
});


var string = new String("str");
var stringTarget = new Proxy(string, {});
var stringProxy = new Proxy(stringTarget, {
  set: null,
});

stringProxy[4] = 1;
assert.sameValue(string[4], 1);

assert(!Reflect.set(stringProxy, "0", "s"));
assert(!Reflect.set(stringProxy, "length", 3));
