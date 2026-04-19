// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-hasproperty-p
description: >
  If "has" trap is null or undefined, [[HasProperty]] call is properly
  forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[HasProperty]] ( P )

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "has").
  7. If trap is undefined, then
    a. Return ? target.[[HasProperty]](P).
features: [Proxy, Symbol.replace, Reflect]
---*/

var regExp = /(?:)/m;
var regExpTarget = new Proxy(regExp, {});
var regExpProxy = new Proxy(regExpTarget, {});

assert(Reflect.has(regExpProxy, "ignoreCase"));
assert(Symbol.replace in regExpProxy);
assert("lastIndex" in Object.create(regExpProxy));


var functionTarget = new Proxy(function() {}, {});
var functionProxy = new Proxy(functionTarget, {});

assert("name" in functionProxy);
assert("length" in Object.create(functionProxy));
