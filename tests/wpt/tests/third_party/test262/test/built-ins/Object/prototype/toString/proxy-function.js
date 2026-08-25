// Copyright (C) 2016 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.tostring
description: Proxy of an function is treated as an function
info: |
  [...]

  9.5.14 ProxyCreate(target, handler)

  [...]
  7. If IsCallable(target) is true, then
    a. Set the [[Call]] internal method of P as specified in 9.5.12.
  [...]

features: [generators, async-functions, Proxy, Symbol.toStringTag]
---*/

var functionProxy = new Proxy(function() {}, {});
var functionProxyProxy = new Proxy(functionProxy, {});

assert.sameValue(
  Object.prototype.toString.call(functionProxy), '[object Function]', 'function proxy'
);
assert.sameValue(
  Object.prototype.toString.call(functionProxyProxy),
  '[object Function]',
  'proxy for function proxy'
);

var arrowProxy = new Proxy(() => {}, {});
var arrowProxyProxy = new Proxy(arrowProxy, {});

assert.sameValue(
  Object.prototype.toString.call(arrowProxy), '[object Function]', 'arrow function proxy'
);
assert.sameValue(
  Object.prototype.toString.call(arrowProxyProxy),
  '[object Function]',
  'proxy for arrow function proxy'
);

var generatorProxy = new Proxy(function*() {}, {});
var generatorProxyProxy = new Proxy(generatorProxy, {});

assert.sameValue(
  Object.prototype.toString.call(generatorProxy), '[object GeneratorFunction]', 'generator function proxy'
);
assert.sameValue(
  Object.prototype.toString.call(generatorProxyProxy),
  '[object GeneratorFunction]',
  'proxy for generator function proxy'
);

var asyncProxy = new Proxy(async function() {}, {});
var asyncProxyProxy = new Proxy(asyncProxy, {});

assert.sameValue(
  Object.prototype.toString.call(asyncProxy), '[object AsyncFunction]', 'async function proxy'
);
assert.sameValue(
  Object.prototype.toString.call(asyncProxyProxy),
  '[object AsyncFunction]',
  'proxy for async function proxy'
);
