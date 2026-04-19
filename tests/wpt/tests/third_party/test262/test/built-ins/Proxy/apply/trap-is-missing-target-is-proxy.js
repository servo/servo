// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist
description: >
  If "apply" trap is null or undefined, [[Call]] is properly
  forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[Call]] ( thisArgument, argumentsList )

  [...]
  4. Let target be O.[[ProxyTarget]].
  5. Let trap be ? GetMethod(handler, "apply").
  6. If trap is undefined, then
    a. Return ? Call(target, thisArgument, argumentsList).
features: [Proxy, Reflect]
---*/

var hasOwn = Object.prototype.hasOwnProperty;
var hasOwnTarget = new Proxy(hasOwn, {});
var hasOwnProxy = new Proxy(hasOwnTarget, {});

var obj = {foo: 1};
assert(hasOwnProxy.call(obj, "foo"));
assert(!Reflect.apply(hasOwnProxy, obj, ["bar"]));
