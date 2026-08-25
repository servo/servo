// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-get-p-receiver
description: >
  If "get" trap is null or undefined, [[Get]] call is properly
  forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[Get]] ( P, Receiver )

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "get").
  7. If trap is undefined, then
    a. Return ? target.[[Get]](P, Receiver).
features: [Proxy, Symbol.match]
---*/

var regExp = /(?:)/i;
var regExpTarget = new Proxy(regExp, {});
var regExpProxy = new Proxy(regExpTarget, {});

assert.sameValue(Object.create(regExpProxy).lastIndex, 0);
assert.sameValue(regExpProxy[Symbol.match], RegExp.prototype[Symbol.match]);


var functionTarget = new Proxy(function(_arg) {}, {});
var functionProxy = new Proxy(functionTarget, {});

assert.sameValue(Object.create(functionProxy).length, 1);
