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

var barValue;
var plainObject = {
  get foo() {},
  set bar(value) {
    barValue = value;
  },
};

var plainObjectTarget = new Proxy(plainObject, {});
var plainObjectProxy = new Proxy(plainObjectTarget, {});

plainObjectProxy.bar = 1;
assert.sameValue(barValue, 1);

assert.throws(TypeError, function() {
  "use strict";
  plainObjectProxy.foo = 2;
});


var regExp = /(?:)/g;
var regExpTarget = new Proxy(regExp, {});
var regExpProxy = new Proxy(regExpTarget, {});

assert(!Reflect.set(regExpProxy, "global", true));

regExpProxy.lastIndex = 1;
assert.sameValue(regExp.lastIndex, 1);
