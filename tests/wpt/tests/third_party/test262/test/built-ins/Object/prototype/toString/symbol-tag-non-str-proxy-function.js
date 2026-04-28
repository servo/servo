// Copyright (C) 2016 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.tostring
description: >
  Non-string values of `Symbol.toStringTag` property are ignored.
info: |
  ProxyCreate ( target, handler )

  [...]
  7. If IsCallable(target) is true, then
    a. Set P.[[Call]] as specified in 9.5.12.

  Object.prototype.toString ( )

  [...]
  7. Else if O has a [[Call]] internal method, let builtinTag be "Function".
  [...]
  15. Let tag be ? Get(O, @@toStringTag).
  16. If Type(tag) is not String, set tag to builtinTag.
  17. Return the string-concatenation of "[object ", tag, and "]".
features: [generators, async-functions, Proxy, Symbol.toStringTag]
---*/

var generatorProxy = new Proxy(function* () {}, {});
var generatorProxyProxy = new Proxy(generatorProxy, {});
delete generatorProxy.constructor.prototype[Symbol.toStringTag];

assert.sameValue(
  Object.prototype.toString.call(generatorProxy),
  '[object Function]',
  'generator function proxy without Symbol.toStringTag'
);
assert.sameValue(
  Object.prototype.toString.call(generatorProxyProxy),
  '[object Function]',
  'proxy for generator function proxy without Symbol.toStringTag'
);

var asyncProxy = new Proxy(async function() {}, {});
var asyncProxyProxy = new Proxy(asyncProxy, {});
Object.defineProperty(asyncProxy.constructor.prototype, Symbol.toStringTag, {
  value: undefined,
});

assert.sameValue(
  Object.prototype.toString.call(asyncProxy),
  '[object Function]',
  'async function proxy without Symbol.toStringTag'
);
assert.sameValue(
  Object.prototype.toString.call(asyncProxyProxy),
  '[object Function]',
  'proxy for async function proxy without Symbol.toStringTag'
);
