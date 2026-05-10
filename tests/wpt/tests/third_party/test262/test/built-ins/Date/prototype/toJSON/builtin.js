// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.tojson
description: >
  Tests that Date.prototype.toJSON meets the requirements
  for built-in objects defined by the introduction of chapter 17 of
  the ECMAScript Language Specification.
features: [Reflect.construct]
---*/

assert(Object.isExtensible(Date.prototype.toJSON), 'Object.isExtensible(Date.prototype.toJSON) must return true');
assert.sameValue(typeof Date.prototype.toJSON, 'function', 'The value of `typeof Date.prototype.toJSON` is "function"');
assert.sameValue(
  Object.prototype.toString.call(Date.prototype.toJSON),
  '[object Function]',
  'Object.prototype.toString.call(Date.prototype.toJSON) must return "[object Function]"'
);
assert.sameValue(
  Object.getPrototypeOf(Date.prototype.toJSON),
  Function.prototype,
  'Object.getPrototypeOf(Date.prototype.toJSON) must return the value of Function.prototype'
);
assert.sameValue(Date.prototype.toJSON.hasOwnProperty('prototype'), false, 'Date.prototype.toJSON.hasOwnProperty("prototype") must return false');
