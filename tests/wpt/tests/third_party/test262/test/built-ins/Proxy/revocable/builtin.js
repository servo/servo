// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy.revocable
description: >
  Requirements for built-in functions, defined in introduction of chapter 17,
  are satisfied.
features: [Proxy, Reflect.construct]
---*/

assert(Object.isExtensible(Proxy.revocable), 'Object.isExtensible(Proxy.revocable) must return true');
assert.sameValue(typeof Proxy.revocable, 'function', 'The value of `typeof Proxy.revocable` is "function"');
assert.sameValue(
  Object.prototype.toString.call(Proxy.revocable),
  '[object Function]',
  'Object.prototype.toString.call(Proxy.revocable) must return "[object Function]"'
);
assert.sameValue(
  Object.getPrototypeOf(Proxy.revocable),
  Function.prototype,
  'Object.getPrototypeOf(Proxy.revocable) must return the value of Function.prototype'
);

assert.sameValue(
  Proxy.revocable.hasOwnProperty('prototype'),
  false,
  'Proxy.revocable.hasOwnProperty(\'prototype\') must return false'
);
