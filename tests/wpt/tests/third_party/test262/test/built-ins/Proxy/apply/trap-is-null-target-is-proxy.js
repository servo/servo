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
features: [Proxy]
---*/

var sum = function(a, b) {
  return this.foo + a + b;
};

var sumBound = sum.bind({foo: 10}, 1);
var sumTarget = new Proxy(sumBound, {});
var sumProxy = new Proxy(sumTarget, {
  apply: null,
});

assert.sameValue(sumProxy(2), 13);
assert.sameValue(sumProxy.call({foo: 20}, 3), 14);
