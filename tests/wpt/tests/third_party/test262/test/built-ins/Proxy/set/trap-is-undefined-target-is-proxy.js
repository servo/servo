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
---*/

var func = function() {};
var funcTarget = new Proxy(func, {});
var funcProxy = new Proxy(funcTarget, {
  set: undefined,
});

assert(Reflect.set(funcProxy, "prototype", null));
assert.sameValue(func.prototype, null);

assert(!Reflect.set(funcProxy, "length", 2));
assert.throws(TypeError, function() {
  "use strict";
  funcProxy.name = "foo";
});


var trapCalls = 0;
var target = new Proxy({}, {
  set: function(_target, key) {
    trapCalls++;
    return key === "foo";
  },
});

var proxy = new Proxy(target, {
  set: undefined,
});

assert(Reflect.set(Object.create(proxy), "foo", 1));
assert.sameValue(trapCalls, 1);

assert(!Reflect.set(proxy, "bar", 2));
assert.sameValue(trapCalls, 2);
