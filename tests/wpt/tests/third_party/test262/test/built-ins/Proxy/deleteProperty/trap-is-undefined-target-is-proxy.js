// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-delete-p
description: >
  If "deleteProperty" trap is null or undefined, [[Delete]] call is
  properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[Delete]] ( P )

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "deleteProperty").
  7. If trap is undefined, then
    a. Return ? target.[[Delete]](P).
features: [Proxy, Reflect]
---*/

var array = [1];
var arrayTarget = new Proxy(array, {});
var arrayProxy = new Proxy(arrayTarget, {
  deleteProperty: undefined,
});

assert(delete arrayProxy[0]);
assert(!array.hasOwnProperty("0"));

assert(!Reflect.deleteProperty(arrayProxy, "length"));
assert.sameValue(array.length, 1);


var trapCalls = 0;
var target = new Proxy({}, {
  deleteProperty: function(_target, key) {
    trapCalls++;
    return key === "foo";
  },
});

var proxy = new Proxy(target, {
  deleteProperty: undefined,
});

assert(delete proxy.foo);
assert.sameValue(trapCalls, 1);

assert.throws(TypeError, function() {
  "use strict";
  delete proxy.bar;
});
assert.sameValue(trapCalls, 2);
