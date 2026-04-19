// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Tests that JSON.stringify meets the requirements
  for built-in objects defined by the introduction of chapter 17 of
  the ECMAScript Language Specification.
features: [Reflect.construct]
---*/

assert(Object.isExtensible(JSON.stringify), 'Object.isExtensible(JSON.stringify) must return true');
assert.sameValue(
  Object.prototype.toString.call(JSON.stringify),
  '[object Function]',
  'Object.prototype.toString.call(JSON.stringify) must return "[object Function]"'
);
assert.sameValue(
  Object.getPrototypeOf(JSON.stringify),
  Function.prototype,
  'Object.getPrototypeOf(JSON.stringify) must return the value of Function.prototype'
);
assert.sameValue(
  JSON.stringify.hasOwnProperty('prototype'),
  false,
  'JSON.stringify.hasOwnProperty("prototype") must return false'
);
