// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-error.prototype-stack
description: >
  Property descriptor of Error.prototype.stack
info: |
  Error.prototype.stack is an accessor property with attributes
  { [[Enumerable]]: false, [[Configurable]]: true }.

  ECMAScript Standard Built-in Objects

  Functions that are specified as get or set accessor functions of built-in
  properties have "get " or "set " prepended to the property name string.

  Unless otherwise specified, the length property of a built-in function object
  has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.

  Unless otherwise specified, the name property of a built-in function object,
  if it exists, has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
features: [error-stack-accessor]
includes: [propertyHelper.js]
---*/

verifyPrimordialAccessorProperty(Error.prototype, 'stack', {
  get: { name: 'get stack', length: 0 },
  set: { name: 'set stack', length: 1 },
}, { label: 'Error.prototype.stack' });
